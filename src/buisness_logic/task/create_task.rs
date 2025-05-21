use super::status_importance::{Importance, Status};
use crate::auth::find_user::Find;
use crate::auth::find_user::FindData;
use crate::models_insertable;
use crate::response::Response as Res;
use crate::schema::tasks::dsl as tasks_table;
use crate::schema::tasks_category as tasks_category_data;
use crate::schema::tasks_category::dsl as tasks_category_table;

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
struct WorkspaceId {
    id: i32,
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    assigner_email: String,
    assignee_email: String,
    description: Option<String>,
    description_multimedia: Option<String>,
    due_date: Option<NaiveDateTime>,
    status: Option<Status>,
    title: String,
    importance: Option<Importance>,
    category: Option<String>,
}

#[post("/workspace/{id}/tasks/create")]
pub async fn create_task(
    pool: DPool,
    req: Json<CreateTaskRequest>,
    id: actix_web::web::Path<WorkspaceId>,
) -> HttpResponse {
    let conn = &mut est_conn(pool.clone());
    let workspace_id = id.id;

    let assigner =
        match <FindData as Find>::find_auth_user_by_email(req.assigner_email.clone(), pool.clone())
            .await
        {
            Ok(user) => user,
            Err(_) => return HttpResponse::BadRequest().json(Res::new("Assigner not found")),
        };

    let assignee =
        match <FindData as Find>::find_auth_user_by_email(req.assignee_email.clone(), pool.clone())
            .await
        {
            Ok(user) => user,
            Err(_) => return HttpResponse::BadRequest().json(Res::new("Assignee not found")),
        };

    let workspaces = match <FindData as Find>::find_workspace_by_owner_email(
        req.assigner_email.clone(),
        pool.clone(),
    )
    .await
    {
        Ok(w) => w,
        Err(_) => {
            return HttpResponse::BadRequest()
                .json(Res::new("Workspace not found for assigner's email"));
        }
    };

    match workspaces.into_iter().find(|w| w.id == workspace_id) {
        Some(_) => {}
        None => {
            return HttpResponse::BadRequest()
                .json(Res::new("Workspace not found for assigner's email"));
        }
    };

    let multimedia_path = if let Some(multimedia_data) = &req.description_multimedia {
        if multimedia_data.is_empty() {
            None
        } else {
            let mut handler = MultimediaHandler::new(multimedia_data.clone(), assigner.id);
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

    let result = conn.transaction::<_, DieselError, _>(|conn| {
        let category_id = match req.category.as_deref() {
            Some(category_name) => {
                match tasks_category_table::tasks_category
                    .filter(tasks_category_data::workspace_id.eq(workspace_id))
                    .filter(tasks_category_data::name.eq(category_name))
                    .select(tasks_category_data::id)
                    .first::<i32>(conn)
                {
                    Ok(id) => id,
                    Err(_) => diesel::insert_into(tasks_category_table::tasks_category)
                        .values((
                            tasks_category_data::workspace_id.eq(workspace_id),
                            tasks_category_data::name.eq(category_name),
                        ))
                        .returning(tasks_category_data::id)
                        .get_result(conn)?,
                }
            }
            None => {
                match tasks_category_table::tasks_category
                    .filter(tasks_category_data::workspace_id.eq(workspace_id))
                    .filter(tasks_category_data::name.eq("NONE"))
                    .select(tasks_category_data::id)
                    .first::<i32>(conn)
                {
                    Ok(id) => id,
                    Err(_) => diesel::insert_into(tasks_category_table::tasks_category)
                        .values((
                            tasks_category_data::workspace_id.eq(workspace_id),
                            tasks_category_data::name.eq("NONE"),
                        ))
                        .returning(tasks_category_data::id)
                        .get_result(conn)?,
                }
            }
        };

        let status_id = req.status.as_ref().map_or(2, |s| match s {
            Status::Todo => 2,
            Status::InProgress => 3,
            Status::Completed => 4,
            Status::Canceled => 5,
            Status::HelpNeeded => 6,
        });

        let importance_id = req.importance.as_ref().map_or(2, |i| match i {
            Importance::Low => 1,
            Importance::Medium => 2,
            Importance::High => 3,
        });

        let new_task = models_insertable::Task {
            workspace_id,
            assigner_id: assigner.id,
            worker_id: assignee.id,
            description: req.description.clone(),
            description_multimedia_path: multimedia_path,
            assignment_date: Utc::now().naive_utc(),
            due_date: req.due_date,
            status_id,
            title: req.title.clone(),
            importance_id,
            category_id,
        };

        diesel::insert_into(tasks_table::tasks)
            .values(&new_task)
            .execute(conn)?;

        Ok(())
    });

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("Task created successfully")),
        Err(DieselError::NotFound) => {
            HttpResponse::BadRequest().json(Res::new("Workspace not found"))
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            HttpResponse::Conflict().json(Res::new("Task with this title already exists"))
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, e)) => {
            eprintln!("FK violation: {:?}", e);
            HttpResponse::BadRequest()
                .json(Res::new("Invalid reference (workspace, user, or category)"))
        }
        Err(err) => {
            eprintln!("Error creating task: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Server error while creating task"))
        }
    }
}
