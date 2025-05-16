use crate::auth::find_user::Find;
use crate::auth::find_user::FindData;
use crate::constants::WORKSPACE_ROLES;
use crate::models;
use crate::models_insertable;
use crate::models_insertable::NewWorkspace;
use crate::response::Response as Res;
use crate::schema::workspace_roles::dsl as workspace_roles_table;
use crate::schema::workspace_users::dsl as workspace_users_table;
use crate::schema::workspaces as workspaces_data;
use crate::schema::workspaces::dsl as workspaces_table;
use actix_web::delete;
use actix_web::{post, web::Json, HttpResponse};
use chrono::NaiveDateTime;
use chrono::Utc;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error as DieselError;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::{Connection, RunQueryDsl};
use serde::Deserialize;

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
