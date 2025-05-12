use crate::auth::find_user::Find;
use crate::auth::find_user::FindData;
use crate::constants::WORKSPACE_ROLES;
use crate::models;
use crate::models_insertable;
use crate::models_insertable::NewWorkspace;
use crate::response::Response as Res;
use crate::schema::workspace_roles::dsl as workspace_roles_table;
use crate::schema::workspace_users::dsl as workspace_users_table;
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

#[post("/workspace/create")]
pub async fn create_workspace(pool: DPool, req: Json<CreateWorkspaceRequest>) -> HttpResponse {
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
            ev_subscription_id: 1,
            name: req.name.clone(),
        };

        // Insert workspace
        let workspace = diesel::insert_into(workspaces_table::workspaces)
            .values(&new_workspace)
            .get_result::<models::Workspace>(conn)?;

        // Add workspace role
        let workspace_role = models_insertable::WorkspaceRole {
            user_id: user.id,
            name: WORKSPACE_ROLES[0].to_string(),
        };

        diesel::insert_into(workspace_roles_table::workspace_roles)
            .values(workspace_role)
            .execute(conn)?;

        let workspace_role = models_insertable::WorkspaceRole {
            user_id: user.id,
            name: WORKSPACE_ROLES[0].to_string(),
        };

        let inserted_role = diesel::insert_into(workspace_roles_table::workspace_roles)
            .values(&workspace_role)
            .get_result::<models::WorkspaceRole>(conn)?;

        let user = models::WorkspaceUser {
            user_id: user.id,
            workspace_id: workspace.id,
            workspace_role_id: inserted_role.id,
            plane_file_cut_name: None,
            position_id: None,
            checkin_time: None,
            checkout_time: None,
        };

        diesel::insert_into(workspace_users_table::workspace_users)
            .values(&user)
            .execute(conn)?;

        Ok(workspace)
    });

    match result {
        Ok(w) => HttpResponse::Ok().json(Res::new(w)),
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, e)) => {
            eprintln!("{:?}", e);
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
            HttpResponse::BadRequest().json(Res::new("Invalid user or subscription reference"))
        }
        Err(err) => {
            eprintln!("Error creating workspace: {}", err);
            HttpResponse::InternalServerError()
                .json(Res::new("Server error while creating workspace"))
        }
    }
}
