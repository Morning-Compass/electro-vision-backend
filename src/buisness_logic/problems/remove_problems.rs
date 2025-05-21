use actix_web::{delete, web::Path, HttpResponse};

use crate::{est_conn, DPool};

use crate::response::Response as Res;
use crate::schema::problems::dsl as problems_table;
use diesel::prelude::*;

#[delete("/problems/{problem_id}/delete")]
pub async fn remove_problem(pool: DPool, path: Path<i32>) -> HttpResponse {
    let problem_id_val = path.into_inner();
    let conn = &mut est_conn(pool);

    let result =
        diesel::delete(problems_table::problems.filter(problems_table::id.eq(problem_id_val)))
            .execute(conn);

    match result {
        Ok(affected) if affected > 0 => {
            HttpResponse::Ok().json(Res::new("Problem deleted successfully"))
        }
        Ok(_) => HttpResponse::NotFound().json(Res::new("Problem not found")),
        Err(err) => {
            eprintln!("Error deleting problem: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Failed to delete problem"))
        }
    }
}
