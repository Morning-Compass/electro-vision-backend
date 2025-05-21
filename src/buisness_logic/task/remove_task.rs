use actix_web::{delete, web::Path, HttpResponse};

use crate::{est_conn, DPool};

use crate::multimedia_handler::{MultimediaHandler, MultimediaHandlerError};
use crate::response::Response as Res;
use crate::schema::tasks::dsl as tasks_table;
use crate::schema::tasks_category as tasks_category_data;
use crate::schema::tasks_category::dsl as tasks_category_table;
use crate::{constants::MAX_MULTIMEDIA_SIZE, schema::tasks as tasks_data};
use actix_web::web::Json;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use serde::Deserialize;

use super::status_importance::{Importance, Status};

#[delete("/workspace/{workspace_id}/tasks/{task_id}/delete")]
pub async fn remove_task(pool: DPool, path: Path<(i32, i32)>) -> HttpResponse {
    let (workspace_id_val, task_id_val) = path.into_inner();
    let conn = &mut est_conn(pool);

    let result = diesel::delete(
        tasks_table::tasks
            .filter(tasks_table::workspace_id.eq(workspace_id_val))
            .filter(tasks_table::id.eq(task_id_val)),
    )
    .execute(conn);

    match result {
        Ok(affected) if affected > 0 => {
            HttpResponse::Ok().json(Res::new("Task deleted successfully"))
        }
        Ok(_) => HttpResponse::NotFound().json(Res::new("Task not found")),
        Err(err) => {
            eprintln!("Error deleting task: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Failed to delete task"))
        }
    }
}
