use crate::auth::find_user::{Find, FindData};
use crate::constants::MAX_MULTIMEDIA_SIZE;
use crate::models::AuthUser;
use crate::multimedia_handler::{MultimediaHandler, MultimediaHandlerError};
use crate::response::Response as Res;
use crate::schema::problems as problems_data;
use crate::schema::problems::dsl as problems_table;
use actix_web::{web::Json, HttpResponse};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
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
struct ProblemChangeset {
    description: Option<String>,
    problem_multimedia_path: Option<Option<String>>,
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

    let mut new_multimedia_path_for_db: Option<Option<String>> = None;

    if let Some(multimedia_data_from_req) = &req.problem_multimedia {
        let existing_multimedia_path: Option<String> = match problems_table::problems
            .filter(problems_data::id.eq(problem_id))
            .filter(problems_data::workspace_id.eq(workspace_id))
            .select(problems_data::problem_multimedia_path)
            .first::<Option<String>>(conn)
        {
            Ok(path_opt) => path_opt,
            Err(DieselError::NotFound) => {
                return HttpResponse::NotFound().json(Res::new("Problem not found"))
            }
            Err(e) => {
                eprintln!("Error fetching problem for multimedia info: {}", e);
                return HttpResponse::InternalServerError().json(Res::new("Database error"));
            }
        };

        if multimedia_data_from_req.is_empty() {
            if let Some(path_to_remove) = existing_multimedia_path {
                let _ = MultimediaHandler::remove_file_by_path(&path_to_remove);
            }
            new_multimedia_path_for_db = Some(None);
        } else {
            let mut handler =
                MultimediaHandler::new(multimedia_data_from_req.to_string(), workspace_id);

            match handler.decode_and_store() {
                Ok(new_file_path_buf) => {
                    let new_file_path_str = new_file_path_buf.to_string_lossy().into_owned();

                    if let Some(old_path) = existing_multimedia_path {
                        if old_path != new_file_path_str {
                            let _ = MultimediaHandler::remove_file_by_path(&old_path);
                        }
                    }
                    new_multimedia_path_for_db = Some(Some(new_file_path_str));
                }
                Err(e) => {
                    eprintln!("Error storing new multimedia: {:?}", e);
                    return match e {
                        MultimediaHandlerError::MaximumFileSizeReached => {
                            HttpResponse::PayloadTooLarge().json(Res::new(format!(
                                "Multimedia file exceeds the size limit ({} MB).",
                                MAX_MULTIMEDIA_SIZE
                            )))
                        }
                        MultimediaHandlerError::DecodingError => HttpResponse::BadRequest()
                            .json(Res::new("Failed to decode multimedia data")),
                        MultimediaHandlerError::FileSystemError => {
                            HttpResponse::InternalServerError()
                                .json(Res::new("Failed to save multimedia file"))
                        }
                        MultimediaHandlerError::InvalidFileType => {
                            HttpResponse::BadRequest().json(Res::new("Unsupported file type"))
                        }
                    };
                }
            }
        }
    } else {
    }

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
            description: req.description.clone(),
            problem_multimedia_path: new_multimedia_path_for_db,
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
