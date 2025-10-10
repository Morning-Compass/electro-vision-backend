use crate::auth::find_user::{Find, FindData};
use crate::response::Response as Res;
use crate::schema::workspaces as workspaces_data;
use crate::schema::workspaces::dsl as workspaces_table;
use actix_web::{web::Json, HttpResponse};
use chrono::NaiveDateTime;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error as DieselError;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::{Connection, RunQueryDsl};
use serde::Deserialize;

use crate::{est_conn, DPool};

#[derive(Deserialize)]
struct UpdateWorkspaceRequest {
    name: Option<String>,
    finish_date: Option<NaiveDateTime>,
    plan_file_name: Option<String>,
    geolocation: Option<String>,
}

#[actix_web::put("/workspace/{workspace_id}")]
pub async fn update_workspace(
    pool: DPool,
    workspace_id: actix_web::web::Path<i32>,
    req: Json<UpdateWorkspaceRequest>,
) -> HttpResponse {
    let conn = &mut est_conn(pool.clone());

    let workspace = match <FindData as Find>::find_workspace_by_id(*workspace_id, pool).await {
        Ok(w) => w,
        Err(e) => match e {
            DieselError::NotFound => {
                return HttpResponse::NotFound().json(Res::new("Workspace not found"))
            }
            _ => {
                return HttpResponse::InternalServerError().json(Res::new(
                    "Internal server error while searching for workspace",
                ))
            }
        },
    };

    let result = conn.transaction::<_, DieselError, _>(|conn| {
        diesel::update(workspaces_table::workspaces.filter(workspaces_data::id.eq(*workspace_id)))
            .set((
                req.name.as_ref().map(|n| workspaces_table::name.eq(n)),
                req.finish_date.map(|d| workspaces_table::finish_date.eq(d)),
                req.plan_file_name
                    .as_ref()
                    .map(|p| workspaces_table::plan_file_name.eq(p)),
                req.geolocation
                    .as_ref()
                    .map(|g| workspaces_table::geolocation.eq(g)),
            ))
            .execute(conn)?;

        Ok(())
    });

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("Workspace updated successfully")),
        Err(DieselError::NotFound) => {
            HttpResponse::NotFound().json(Res::new("Workspace not found"))
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, e)) => {
            eprintln!("Unique constraint violation: {:?}", e);
            if e.message().contains("owner_id_plan_file_name") {
                HttpResponse::Conflict()
                    .json(Res::new("Plan file name already exists for this owner"))
            } else if e.message().contains("owner_id_name") {
                HttpResponse::Conflict()
                    .json(Res::new("Workspace name already exists for this owner"))
            } else {
                HttpResponse::Conflict().json(Res::new("Unique constraint violation"))
            }
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _)) => {
            HttpResponse::BadRequest().json(Res::new("Invalid subscription reference"))
        }
        Err(err) => {
            eprintln!("Error updating workspace: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Failed to update workspace"))
        }
    }
}
