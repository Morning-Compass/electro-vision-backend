use crate::auth::find_user::Find;
use crate::auth::find_user::FindData;
use crate::models_insertable;
use crate::response::Response as Res;
use crate::schema::problems as problems_data;
use crate::schema::problems::dsl as problems_table; // Import problems dsl
use crate::schema::tasks_category as tasks_category_data;
use crate::schema::tasks_category::dsl as tasks_category_table; // Import for AsChangeset

use actix_web::{post, web::Json, HttpResponse};
use chrono::NaiveDateTime;
use chrono::Utc;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error as DieselError;
use diesel::QueryDsl;
use diesel::{Connection, ExpressionMethods, RunQueryDsl};
use serde::Deserialize;

use crate::constants::MAX_MULTIMEDIA_SIZE;
use crate::multimedia_handler::{MultimediaHandler, MultimediaHandlerError};
use crate::{est_conn, DPool};

#[derive(Deserialize)]
struct CreateProblemRequest {
    worker_email: String,
    mentor_email: String,
    description: Option<String>,
    problem_multimedia: Option<String>,
}

#[derive(Deserialize)]
pub struct WorkspaceId {
    id: i32,
}

#[post("/workspace/{id}/problems/create")]
pub async fn create_problem(
    pool: DPool,
    req: Json<CreateProblemRequest>,
    id: actix_web::web::Path<WorkspaceId>, // Re-using WorkspaceId from tasks for consistency
) -> HttpResponse {
    let conn = &mut est_conn(pool.clone());
    let workspace_id = id.id; // Assuming problems are associated with a workspace

    let worker =
        match <FindData as Find>::find_auth_user_by_email(req.worker_email.clone(), pool.clone())
            .await
        {
            Ok(user) => user,
            Err(_) => return HttpResponse::BadRequest().json(Res::new("Worker not found")),
        };

    let mentor =
        match <FindData as Find>::find_auth_user_by_email(req.mentor_email.clone(), pool.clone())
            .await
        {
            Ok(user) => user,
            Err(_) => return HttpResponse::BadRequest().json(Res::new("Mentor not found")),
        };

    let workspaces =
        match <FindData as Find>::find_workspace_by_id(workspace_id.clone(), pool.clone()).await {
            Ok(w) => w,
            Err(_) => {
                return HttpResponse::BadRequest()
                    .json(Res::new("Workspace not found for that id"));
            }
        };

    match workspaces.into_iter().find(|w| w.id == workspace_id) {
        Some(_) => {}
        None => {
            return HttpResponse::BadRequest().json(Res::new("Workspace not found for that id"));
        }
    };

    let multimedia_path = if let Some(multimedia_data) = &req.problem_multimedia {
        if multimedia_data.is_empty() {
            None
        } else {
            let mut handler = MultimediaHandler::new(multimedia_data.clone(), worker.id);
            match handler.decode_and_store() {
                Ok(_) => handler.get_file_path(),
                Err(MultimediaHandlerError::MaximumFileSizeReached) => {
                    return HttpResponse::PayloadTooLarge().json(Res::new(format!(
                        "Multimedia file exceeds the size limit ({} MB).",
                        MAX_MULTIMEDIA_SIZE
                    )));
                }
                Err(MultimediaHandlerError::DecodingError) => {
                    return HttpResponse::InternalServerError()
                        .json(Res::new("Failed to decode multimedia data."));
                }
                Err(MultimediaHandlerError::FileSystemError) => {
                    return HttpResponse::InternalServerError().json(Res::new(
                        "A file system error occurred while saving the file.",
                    ));
                }
                Err(MultimediaHandlerError::InvalidFileType) => {
                    return HttpResponse::UnsupportedMediaType()
                        .json(Res::new("Unsupported multimedia file type."));
                }
            }
        }
    } else {
        None
    };

    let new_problem = models_insertable::Problem {
        worker_id: worker.id,
        mentor_id: mentor.id,
        workspace_id,
        description: req.description.clone(),
        problem_multimedia_path: multimedia_path,
    };

    let result = conn.transaction::<_, DieselError, _>(|c| {
        diesel::insert_into(problems_table::problems)
            .values(&new_problem)
            .execute(c)?;
        Ok(())
    });

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("Problem created successfully")),
        Err(DieselError::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, e)) => {
            eprintln!("FK violation: {:?}", e);
            HttpResponse::BadRequest().json(Res::new("Invalid worker or mentor ID"))
        }
        Err(err) => {
            eprintln!("Error creating problem: {}", err);
            HttpResponse::InternalServerError()
                .json(Res::new("Server error while creating problem"))
        }
    }
}
