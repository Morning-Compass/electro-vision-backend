use actix_web::{delete, web::Path, HttpResponse};

use crate::models::Task;
use crate::{est_conn, DPool};

use crate::response::Response as Res;
use crate::schema::tasks::dsl as tasks_table;
use diesel::prelude::*;

#[delete("/workspace/{workspace_id}/tasks/{task_id}/delete")]
pub async fn remove_task(pool: DPool, path: Path<(i32, i32)>) -> HttpResponse {
    let (workspace_id_val, task_id_val) = path.into_inner();
    let conn = &mut est_conn(pool);

    let result = diesel::delete(
        tasks_table::tasks
            .filter(tasks_table::workspace_id.eq(workspace_id_val))
            .filter(tasks_table::id.eq(task_id_val)),
    )
    .get_result::<Task>(conn);

    match result {
        Ok(task) => HttpResponse::Ok().json(Res::new(task)),
        Err(err) => {
            eprintln!("Error deleting task: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Failed to delete task"))
        }
    }
}
