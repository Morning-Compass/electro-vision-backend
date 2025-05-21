use crate::auth::find_user::{Find, FindData};
use crate::constants::MAX_MULTIMEDIA_SIZE;
use crate::models::AuthUser;
use crate::multimedia_handler::{MultimediaHandler, MultimediaHandlerError};
use crate::response::Response as Res;
use crate::schema::problems as problems_data; // Import for AsChangeset
use crate::schema::problems::dsl as problems_table; // Import problems dsl
use actix_web::{web::Json, HttpResponse};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use serde::Deserialize;

use crate::{est_conn, DPool};

#[derive(Deserialize)]
struct UpdateProblemRequest {
    description: Option<String>,
    problem_multimedia: Option<String>,
    worker_email: Option<String>,
    mentor_email: Option<String>,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = problems_data)]
struct ProblemChangeset<'a> {
    description: Option<&'a str>,
    problem_multimedia_path: Option<&'a str>,
    worker_id: Option<i32>,
    mentor_id: Option<i32>,
}

#[actix_web::put("/workspace/{workspace_id}/problems/{problem_id}")]
pub async fn update_problem(
    pool: DPool,
    path: actix_web::web::Path<(i32, i32)>,
    req: Json<UpdateProblemRequest>,
) -> HttpResponse {
    let (workspace_id, problem_id) = path.into_inner();
    let conn = &mut est_conn(pool.clone());

    let multimedia_path = if let Some(multimedia_data) = &req.problem_multimedia {
        if multimedia_data.is_empty() {
            None
        } else {
            let worker_id = match problems_table::problems
                .filter(problems_data::id.eq(problem_id))
                .select(problems_data::worker_id)
                .first::<i32>(conn)
            {
                Ok(id) => id,
                Err(_) => return HttpResponse::NotFound().json(Res::new("Problem not found")),
            };

            let mut handler = MultimediaHandler::new(multimedia_data.to_string(), workspace_id);
            match handler.decode_and_store() {
                Ok(_) => handler.get_file_path(),
                Err(MultimediaHandlerError::MaximumFileSizeReached) => {
                    return HttpResponse::PayloadTooLarge().json(Res::new(format!(
                        "Multimedia file exceeds the size limit ({} MB).",
                        MAX_MULTIMEDIA_SIZE
                    )));
                }
                Err(MultimediaHandlerError::DecodingError) => {
                    return HttpResponse::BadRequest()
                        .json(Res::new("Failed to decode multimedia data"));
                }
                Err(MultimediaHandlerError::FileSystemError) => {
                    return HttpResponse::InternalServerError()
                        .json(Res::new("Failed to save multimedia file"));
                }
                Err(MultimediaHandlerError::InvalidFileType) => {
                    return HttpResponse::BadRequest().json(Res::new("Unsupported file type"));
                }
            }
        }
    } else {
        None
    };

    let worker: Option<AuthUser> = match req.worker_email.clone() {
        Some(email) => {
            match <FindData as Find>::find_auth_user_by_email(email.clone(), pool.clone()).await {
                Ok(user) => Some(user),
                Err(_) => return HttpResponse::BadRequest().json(Res::new("Worker not found")),
            }
        }
        None => None,
    };

    let mentor: Option<AuthUser> = match req.mentor_email.clone() {
        Some(email) => {
            match <FindData as Find>::find_auth_user_by_email(email.clone(), pool.clone()).await {
                Ok(user) => Some(user),
                Err(_) => return HttpResponse::BadRequest().json(Res::new("Mentor not found")),
            }
        }
        None => None,
    };

    let result = conn.transaction::<_, DieselError, _>(|conn| {
        let changeset = ProblemChangeset {
            description: req.description.as_deref(),
            problem_multimedia_path: multimedia_path.as_deref(),
            worker_id: worker.map(|u| u.id),
            mentor_id: mentor.map(|u| u.id),
        };

        let affected_rows = diesel::update(
            problems_table::problems
                .filter(problems_data::id.eq(problem_id))
                .filter(problems_data::workspace_id.eq(workspace_id)),
        )
        .set(changeset)
        .execute(conn)?;

        if affected_rows == 0 {
            Err(DieselError::NotFound)
        } else {
            Ok(())
        }
    });

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("Problem updated successfully")),
        Err(DieselError::NotFound) => HttpResponse::NotFound().json(Res::new("Problem not found")),
        Err(err) => {
            eprintln!("Error updating problem: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Failed to update problem"))
        }
    }
}
