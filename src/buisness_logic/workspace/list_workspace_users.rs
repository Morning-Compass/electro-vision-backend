use crate::auth::find_user::{Find, FindData};
use crate::response::Response as Res;
use crate::schema::auth_users::dsl as auth_users_table;
use crate::schema::positions as positions_data;
use crate::schema::positions::dsl as positions_table;
use crate::schema::workspace_roles as workspace_roles_data;
use crate::schema::workspace_roles::dsl as workspace_roles_table;
use crate::schema::workspace_users as workspace_users_data;
use crate::schema::workspace_users::dsl as workspace_users_table;
use crate::schema::workspaces as workspaces_data;
use crate::schema::workspaces::dsl as workspaces_table;
use crate::{schema::auth_users as auth_users_data, DBPConn};

use actix_web::{
    post,
    web::{Json, Path},
    HttpResponse,
};
use diesel::{result::Error as DieselError, ExpressionMethods};
use diesel::{JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};

use crate::{est_conn, DPool};

#[derive(Deserialize)]
struct ListWorkspaceUsersRequest {
    email: String,
}

#[derive(Deserialize)]
struct ListWorkspaceUsersPath {
    id: i32,
}

#[derive(Serialize)]
struct WorkspaceUserResponse {
    id: i32,
    username: String,
    email: String,
    position: Option<String>,
    workspace_role: String,
}

#[post("/workspace/{id}/users/list")]
pub async fn list_workspace_users(
    pool: DPool,
    req: Json<ListWorkspaceUsersRequest>,
    path: Path<ListWorkspaceUsersPath>,
) -> HttpResponse {
    let workspace_id = path.id;
    let _email = req.email.clone();

    let conn = &mut est_conn(pool.clone());

    let workspaces =
        match <FindData as Find>::find_workspace_by_owner_email(req.email.clone(), pool.clone())
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

    match get_workspace_users(conn, workspace_id).await {
        Ok(users) => HttpResponse::Ok().json(Res::new(users)),
        Err(DieselError::NotFound) => {
            HttpResponse::NotFound().json(Res::new("Workspace not found for that id"))
        }
        Err(err) => {
            eprintln!("Error listing workspace users: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Server error while listing users"))
        }
    }
}

async fn get_workspace_users(
    conn: &mut DBPConn,
    workspace_id: i32,
) -> Result<Vec<WorkspaceUserResponse>, DieselError> {
    let users = workspace_users_table::workspace_users
        .filter(workspace_users_data::workspace_id.eq(workspace_id))
        .inner_join(auth_users_table::auth_users)
        .left_join(
            positions_table::positions.on(workspace_users_data::position_id
                .eq(positions_data::id.nullable())
                .nullable()),
        )
        .inner_join(
            workspace_roles_table::workspace_roles
                .on(workspace_users_data::workspace_role_id.eq(workspace_roles_data::id)),
        )
        .select((
            auth_users_data::id,
            auth_users_data::username,
            auth_users_data::email,
            positions_data::name.nullable(),
            workspace_roles_data::name,
        ))
        .load::<(i32, String, String, Option<String>, String)>(conn)?
        .into_iter()
        .map(
            |(id, username, email, position, workspace_role)| WorkspaceUserResponse {
                id,
                username,
                email,
                position,
                workspace_role,
            },
        )
        .collect();

    Ok(users)
}
