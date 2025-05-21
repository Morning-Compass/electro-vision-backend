use crate::response::Response as Res;
use crate::schema::workspaces::dsl as workspaces_table;
use actix_web::delete;
use actix_web::HttpResponse;
use diesel::result::Error as DieselError;
use diesel::QueryDsl;
use diesel::{Connection, RunQueryDsl};

use crate::{est_conn, DPool};

#[delete("/workspace/{workspace_id}")]
pub async fn delete_workspace(
    pool: DPool,
    workspace_id: actix_web::web::Path<i32>,
) -> HttpResponse {
    let conn = &mut est_conn(pool);

    let result = conn.transaction::<_, DieselError, _>(|conn| {
        diesel::delete(workspaces_table::workspaces.find(*workspace_id)).execute(conn)?;

        Ok(())
    });

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("Workspace Deleted successfully")),
        Err(DieselError::NotFound) => {
            HttpResponse::NotFound().json(Res::new("Workspace not found"))
        }
        Err(err) => {
            eprintln!("Error deleting workspace: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Failed to delete workspace"))
        }
    }
}
