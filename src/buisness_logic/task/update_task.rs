use crate::multimedia_handler::{MultimediaHandler, MultimediaHandlerError};
use crate::response::Response as Res;
use crate::schema::tasks::dsl as tasks_table;
use crate::schema::tasks_category as tasks_category_data;
use crate::schema::tasks_category::dsl as tasks_category_table;
use crate::{constants::MAX_MULTIMEDIA_SIZE, schema::tasks as tasks_data};
use actix_web::{web::Json, HttpResponse};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use serde::Deserialize;

use crate::{est_conn, DPool};

use super::status_importance::{Importance, Status};

#[derive(Deserialize)]
struct UpdateTaskRequest {
    title: Option<String>,
    description: Option<String>,
    description_multimedia: Option<String>,
    due_date: Option<NaiveDateTime>,
    status: Option<Status>,
    importance: Option<Importance>,
    category: Option<String>,
}

// Define a changeset struct
#[derive(AsChangeset, Default)]
#[diesel(table_name = tasks_data)]
struct TaskChangeset<'a> {
    title: Option<&'a str>,
    description: Option<&'a str>,
    description_multimedia_path: Option<&'a str>,
    due_date: Option<NaiveDateTime>,
    status_id: Option<i32>,
    importance_id: Option<i32>,
    category_id: Option<i32>,
}

#[actix_web::put("/workspace/{workspace_id}/tasks/{task_id}")]
pub async fn update_task(
    pool: DPool,
    path: actix_web::web::Path<(i32, i32)>,
    req: Json<UpdateTaskRequest>,
) -> HttpResponse {
    let (workspace_id, task_id) = path.into_inner();
    let conn = &mut est_conn(pool.clone());

    // Handle multimedia if provided
    let multimedia_path = if let Some(multimedia_data) = &req.description_multimedia {
        if multimedia_data.is_empty() {
            None
        } else {
            // Get assigner_id for the task to use in multimedia path
            let assigner_id = match tasks_table::tasks
                .filter(tasks_data::id.eq(task_id))
                .select(tasks_data::assigner_id)
                .first::<i32>(conn)
            {
                Ok(id) => id,
                Err(_) => return HttpResponse::NotFound().json(Res::new("Task not found")),
            };

            let mut handler = MultimediaHandler::new(multimedia_data.to_string(), assigner_id);
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

    let result = conn.transaction::<_, DieselError, _>(|conn| {
        // Get or create category if provided
        let category_id = if let Some(category_name) = &req.category {
            match tasks_category_table::tasks_category
                .filter(tasks_category_data::workspace_id.eq(workspace_id))
                .filter(tasks_category_data::name.eq(category_name))
                .select(tasks_category_data::id)
                .first::<i32>(conn)
                .optional()?
            {
                Some(id) => Some(id),
                None => Some(
                    diesel::insert_into(tasks_category_table::tasks_category)
                        .values((
                            tasks_category_data::workspace_id.eq(workspace_id),
                            tasks_category_data::name.eq(category_name),
                        ))
                        .returning(tasks_category_data::id)
                        .get_result(conn)?,
                ),
            }
        } else {
            None
        };

        // Map status and importance to IDs
        let status_id = req.status.as_ref().map(|s| match s {
            Status::HelpNeeded => 1,
            Status::Todo => 2,
            Status::InProgress => 3,
            Status::Completed => 4,
            Status::Canceled => 5,
        });

        let importance_id = req.importance.as_ref().map(|i| match i {
            Importance::Low => 1,
            Importance::Medium => 2,
            Importance::High => 3,
        });

        // Build the changeset
        let changeset = TaskChangeset {
            title: req.title.as_deref(),
            description: req.description.as_deref(),
            description_multimedia_path: multimedia_path.as_deref(),
            due_date: req.due_date,
            status_id,
            importance_id,
            category_id,
        };

        // Execute update
        let affected_rows = diesel::update(
            tasks_table::tasks
                .filter(tasks_data::id.eq(task_id))
                .filter(tasks_data::workspace_id.eq(workspace_id)),
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
        Ok(_) => HttpResponse::Ok().json(Res::new("Task updated successfully")),
        Err(DieselError::NotFound) => HttpResponse::NotFound().json(Res::new("Task not found")),
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            HttpResponse::Conflict().json(Res::new("Task with this title already exists"))
        }
        Err(err) => {
            eprintln!("Error updating task: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Failed to update task"))
        }
    }
}
