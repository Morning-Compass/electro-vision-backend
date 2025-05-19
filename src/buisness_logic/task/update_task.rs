use std::io;

use super::status_importance::{Importance, Status};
use crate::auth::find_user::FindData;
use crate::multimedia_handler::MultimediaHandler;
use crate::response::Response as Res;
use crate::schema::importance;
use crate::schema::tasks as tasks_data;
use crate::schema::tasks::dsl as tasks_table;
use crate::schema::tasks_category as tasks_category_data;
use crate::schema::tasks_category::dsl as tasks_category_table;
use crate::schema::workspaces as workspaces_data;
use crate::schema::workspaces::dsl as workspaces_table;
use crate::DPool;
use crate::{auth::find_user::Find, est_conn};
use actix_web::post;
use actix_web::{
    web::{Json, Path},
    HttpResponse,
};
use base64::{engine::general_purpose, Engine as _};
use chrono::NaiveDateTime;
use diesel::query_dsl::methods::FilterDsl;
use diesel::result::Error;
use diesel::result::Error as DieselError;
use diesel::Connection;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use mime_guess::MimeGuess;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct UpdateTaskRequest {
    title: Option<String>,
    description: Option<String>,
    due_date: Option<NaiveDateTime>,
    status: Option<Status>,
    importance: Option<Importance>,
    category: Option<String>,
}

#[actix_web::put("/workspace/{workspace_id}/tasks/{task_id}/update")]
pub async fn update_task(
    pool: DPool,
    path: Path<(i32, i32)>,
    req: Json<UpdateTaskRequest>,
) -> HttpResponse {
    let (workspace_id_val, task_id_val) = path.into_inner();
    let req_data = req.into_inner(); // consume for Send safety
    let conn = &mut est_conn(pool.clone());

    let result = conn.transaction::<_, DieselError, _>(|conn| {
        let category_id_val = if let Some(ref cat_name) = req_data.category {
            match tasks_category_table::tasks_category
                .filter(tasks_category_data::workspace_id.eq(workspace_id_val))
                .filter(tasks_category_data::name.eq(cat_name))
                .select(tasks_category_data::id)
                .first::<i32>(conn)
            {
                Ok(id) => Some(id),
                Err(_) => Some(
                    diesel::insert_into(tasks_category_table::tasks_category)
                        .values((
                            tasks_category_data::workspace_id.eq(workspace_id_val),
                            tasks_category_data::name.eq(cat_name),
                        ))
                        .returning(tasks_category_data::id)
                        .get_result(conn)?,
                ),
            }
        } else {
            None
        };

        let status_id_val = req_data.status.map(|s| match s {
            Status::HelpNeeded => 1,
            Status::Todo => 2,
            Status::InProgress => 3,
            Status::Completed => 4,
            Status::Canceled => 5,
        });

        let importance_id_val = req_data.importance.map(|i| match i {
            Importance::Low => 1,
            Importance::Medium => 2,
            Importance::High => 3,
        });

        let mut query = diesel::update(
            tasks_table::tasks
                .filter(tasks_data::id.eq(task_id_val))
                .filter(tasks_data::workspace_id.eq(workspace_id_val)),
        )
        .into_boxed();

        if let Some(ref t) = req_data.title {
            query = query.set(tasks_data::title.eq(t));
        }
        if let Some(ref d) = req_data.description {
            query = query.set(tasks_data::description.eq(d));
        }
        if let Some(due) = req_data.due_date {
            query = query.set(tasks_data::due_date.eq(due));
        }
        if let Some(sid) = status_id_val {
            query = query.set(tasks_data::status_id.eq(sid));
        }
        if let Some(iid) = importance_id_val {
            query = query.set(tasks_data::importance_id.eq(iid));
        }
        if let Some(cid) = category_id_val {
            query = query.set(tasks_data::category_id.eq(cid));
        }

        let affected = query.execute(conn)?;

        if affected == 0 {
            Err(DieselError::NotFound)
        } else {
            Ok(())
        }
    });

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("Task updated successfully")),
        Err(DieselError::NotFound) => HttpResponse::NotFound().json(Res::new("Task not found")),
        Err(err) => {
            eprintln!("Error updating task: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Failed to update task"))
        }
    }
}
