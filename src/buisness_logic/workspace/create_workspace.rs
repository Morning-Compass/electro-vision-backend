use crate::auth::find_user::Find;
use crate::auth::find_user::FindData;
use crate::constants::WORKSPACE_ROLES;
use crate::models_insertable;
use crate::models_insertable::NewWorkspace;
use crate::response::Response as Res;
use crate::schema::workspace_roles::dsl as workspace_roles_table;
use crate::schema::workspaces::dsl as workspaces_table;
use actix_web::{post, web::Json, HttpResponse};
use chrono::NaiveDateTime;
use chrono::Utc;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error as DieselError;
use diesel::{Connection, RunQueryDsl};
use serde::Deserialize;

use crate::{est_conn, DPool};

#[derive(Deserialize)]
struct CreateWorkspaceRequest {
    owner_email: String,
    name: String,
    finish_date: Option<NaiveDateTime>,
    plan_file_name: Option<String>,
    geolocation: Option<String>,
}

#[post("/create_workspace")]
pub async fn create_workspace(pool: DPool, req: Json<CreateWorkspaceRequest>) -> HttpResponse {
    // Find user first to handle foreign key constraint
    let user =
        match <FindData as Find>::find_auth_user_by_email(req.owner_email.clone(), pool.clone())
            .await
        {
            Ok(user) => user,
            Err(_) => {
                return HttpResponse::BadRequest().json(Res::new("User not found"));
            }
        };

    let conn = &mut est_conn(pool);

    let result = conn.transaction::<_, DieselError, _>(|conn| {
        let new_workspace = NewWorkspace {
            owner_id: user.id.clone(),
            geolocation: req.geolocation.as_deref(),
            plan_file_name: req.plan_file_name.as_deref(),
            start_date: Utc::now().naive_utc(),
            finish_date: req.finish_date,
            ev_subscription_id: 1, // You might want to validate this exists too
            name: req.name.clone(),
        };

        // Insert workspace
        diesel::insert_into(workspaces_table::workspaces)
            .values(&new_workspace)
            .execute(conn)?;

        // Add workspace role
        let workspace_role = models_insertable::WorkspaceRole {
            user_id: user.id,
            name: WORKSPACE_ROLES[0].to_string(),
        };

        diesel::insert_into(workspace_roles_table::workspace_roles)
            .values(workspace_role)
            .execute(conn)?;

        Ok(())
    });

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("Workspace created successfully")),
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            HttpResponse::Conflict().json(Res::new("Workspace name already exists for this user"))
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _)) => {
            HttpResponse::BadRequest().json(Res::new("Invalid user or subscription reference"))
        }
        Err(err) => {
            eprintln!("Error creating workspace: {}", err);
            HttpResponse::InternalServerError()
                .json(Res::new("Server error while creating workspace"))
        }
    }
}
