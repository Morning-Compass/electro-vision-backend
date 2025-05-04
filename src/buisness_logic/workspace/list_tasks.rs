use crate::auth::find_user::FindData;
use crate::response::Response as Res;
use crate::DPool;
use crate::{auth::find_user::Find, est_conn};
use actix_web::post;
use actix_web::{
    get,
    web::{Json, Path},
    HttpResponse,
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::result::Error as DieselError;
use serde::{Deserialize, Serialize};

#[derive(QueryableByName, Serialize)]
struct TaskResponse {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    title: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    due_date: Option<NaiveDateTime>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    importance: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    assigner_username: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    assignee_username: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    category: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamp)]
    created_at: NaiveDateTime,
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

    let workspaces = match <FindData as Find>::find_workspace_by_owner_email(
        req.owner_email.clone(),
        pool.clone(),
    )
    .await
    {
        Ok(w) => w,
        Err(_) => {
            return HttpResponse::BadRequest()
                .json(Res::new("Workspace not found for owner's email or id"));
        }
    };

    match workspaces.into_iter().find(|w| w.id == workspace_id) {
        Some(_) => {}
        None => {
            return HttpResponse::BadRequest()
                .json(Res::new("Workspace not found for owner's email or id"));
        }
    };

    let conn = &mut est_conn(pool);

    let result = conn.transaction::<_, Error, _>(|conn| {
        let query = r#"
            SELECT
                tasks.id,
                tasks.title,
                tasks.description,
                tasks.due_date,
                status.name as status,
                importance.name as importance,
                assigner.username as assigner_username,
                assignee.username as assignee_username,
                tasks_category.name as category,
                tasks.assignment_date as created_at
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
            .load::<TaskResponse>(conn)
    });

    match result {
        Ok(tasks) => HttpResponse::Ok().json(tasks),
        Err(DieselError::NotFound) => HttpResponse::NotFound().json(Res::new("No tasks found")),
        Err(err) => {
            eprintln!("Error listing tasks: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Server error while listing tasks"))
        }
    }
}
