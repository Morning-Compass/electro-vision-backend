use crate::auth::find_user::FindData;
use crate::buisness_logic::task::db_task::DbTask;
use crate::multimedia_handler::{MultimediaHandler, MultimediaHandlerError};
use crate::response::Response as Res;
use crate::DPool;
use crate::{auth::find_user::Find, est_conn};
use actix_web::post;
use actix_web::{
    web::{Json, Path},
    HttpResponse,
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::result::Error as DieselError;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct TaskResponse {
    id: i32,
    title: String,
    description: Option<String>,
    description_multimedia: Option<String>,
    description_multimedia_filename: Option<String>,
    due_date: Option<NaiveDateTime>,
    status: String,
    importance: String,
    assigner_email: String,
    assignee_email: String,
    category: Option<String>,
    created_at: NaiveDateTime,
    task_type: Option<String>,
}

#[derive(Deserialize)]
pub struct WorkspaceId {
    id: i32,
}

#[derive(Deserialize)]
pub struct ListTasksRequest {
    owner_email: String,
}

#[post("/workspace/{id}/tasks/list")]
pub async fn list_tasks(
    pool: DPool,
    path: Path<WorkspaceId>,
    req: Json<ListTasksRequest>,
) -> HttpResponse {
    let workspace_id = path.id;
    let requester_email = req.owner_email.clone();

    // Step 1: Check if requester is the owner of the workspace
    let is_owner = match <FindData as Find>::find_workspace_by_owner_email(
        requester_email.clone(),
        pool.clone(),
    )
    .await
    {
        Ok(workspaces) => workspaces.iter().any(|w| w.id == workspace_id),
        Err(_) => false,
    };

    let conn = &mut est_conn(pool);

    let result = conn.transaction::<_, Error, _>(|conn| {
        // Step 2: Adjust SQL based on ownership
        let (query, binds): (&str, Vec<diesel::sql_types::Untyped>);

        if is_owner {
            // Query all tasks in the workspace
            query = r#"
                SELECT
                    tasks.id,
                    tasks.title,
                    tasks.description,
                    tasks.due_date,
                    status.name as status,
                    importance.name as importance,
                    assigner.email as assigner_username,
                    assignee.email as assignee_username,
                    tasks_category.name as category,
                    tasks.assignment_date as created_at,
                    tasks.description_multimedia_path,
                    tasks.task_type
                FROM tasks
                JOIN status ON tasks.status_id = status.id
                JOIN importance ON tasks.importance_id = importance.id
                JOIN auth_users assigner ON tasks.assigner_id = assigner.id
                JOIN auth_users assignee ON tasks.worker_id = assignee.id
                LEFT JOIN tasks_category ON tasks.category_id = tasks_category.id
                WHERE tasks.workspace_id = $1
                ORDER BY tasks.assignment_date DESC
            "#;
            diesel::sql_query(query)
                .bind::<diesel::sql_types::Integer, _>(workspace_id)
                .load::<DbTask>(conn)
        } else {
            // Query only tasks assigned to this user in the workspace
            query = r#"
                SELECT
                    tasks.id,
                    tasks.title,
                    tasks.description,
                    tasks.due_date,
                    status.name as status,
                    importance.name as importance,
                    assigner.email as assigner_username,
                    assignee.email as assignee_username,
                    tasks_category.name as category,
                    tasks.assignment_date as created_at,
                    tasks.description_multimedia_path,
                    tasks.task_type
                FROM tasks
                JOIN status ON tasks.status_id = status.id
                JOIN importance ON tasks.importance_id = importance.id
                JOIN auth_users assigner ON tasks.assigner_id = assigner.id
                JOIN auth_users assignee ON tasks.worker_id = assignee.id
                LEFT JOIN tasks_category ON tasks.category_id = tasks_category.id
                WHERE tasks.workspace_id = $1 AND assignee.email = $2
                ORDER BY tasks.assignment_date DESC
            "#;
            diesel::sql_query(query)
                .bind::<diesel::sql_types::Integer, _>(workspace_id)
                .bind::<diesel::sql_types::Text, _>(requester_email)
                .load::<DbTask>(conn)
        }
    });

    // Same response formatting logic
    match result {
        Ok(tasks) => {
            let mut res: Vec<TaskResponse> = Vec::new();

            for task in tasks {
                let (multimedia, filename) = match task.description_multimedia_path {
                    Some(ref path) => match MultimediaHandler::get_file_content_base64(path) {
                        Ok(content) => {
                            let filename = std::path::Path::new(path)
                                .file_name()
                                .and_then(|f| f.to_str())
                                .map(|s| s.to_string());
                            (Some(content), filename)
                        }
                        Err(e) => {
                            return match e {
                                MultimediaHandlerError::DecodingError => {
                                    HttpResponse::InternalServerError()
                                        .json(Res::new("Decoding error"))
                                }
                                MultimediaHandlerError::InvalidFileType => {
                                    HttpResponse::InternalServerError()
                                        .json(Res::new("Invalid file type"))
                                }
                                MultimediaHandlerError::MaximumFileSizeReached => {
                                    HttpResponse::InternalServerError()
                                        .json(Res::new("Maximum file size reached"))
                                }
                                MultimediaHandlerError::FileSystemError => {
                                    HttpResponse::InternalServerError()
                                        .json(Res::new("File system error"))
                                }
                            };
                        }
                    },
                    None => (None, None),
                };

                res.push(TaskResponse {
                    id: task.id,
                    title: task.title,
                    description: task.description,
                    description_multimedia: multimedia,
                    description_multimedia_filename: filename,
                    due_date: task.due_date,
                    status: task.status,
                    importance: task.importance,
                    assigner_email: task.assigner_username,
                    assignee_email: task.assignee_username,
                    category: task.category,
                    created_at: task.created_at,
                    task_type: task.task_type,
                });
            }

            HttpResponse::Ok().json(Res::new(res))
        }
        Err(DieselError::NotFound) => HttpResponse::NotFound().json(Res::new("No tasks found")),
        Err(err) => {
            eprintln!("Error listing tasks: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Server error while listing tasks"))
        }
    }
}
