use crate::response::Response as Res;
use crate::schema::auth_users::dsl as auth_users_table;
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
use diesel::{QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};

use crate::{est_conn, DPool};

#[derive(Deserialize)]
struct ListWorkspaceUsersRequest {
    email: String,
}

#[derive(Deserialize)]
struct ListWorkspaceUsersPath {
    id: i32, // Changed to i32 since workspace ID is numeric
}

#[derive(Serialize)]
struct WorkspaceUserResponse {
    id: i32,
    username: String,
    email: String,
}

#[post("/list/workspace/users/{id}")]
pub async fn list_workspace_users(
    pool: DPool,
    req: Json<ListWorkspaceUsersRequest>,
    path: Path<ListWorkspaceUsersPath>,
) -> HttpResponse {
    let workspace_id = path.id;
    let _email = req.email.clone();

    let conn = &mut est_conn(pool);

    match get_workspace_users(conn, workspace_id).await {
        Ok(users) => HttpResponse::Ok().json(Res::new(users)),
        Err(DieselError::NotFound) => {
            HttpResponse::NotFound().json(Res::new("Workspace not found"))
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
    let workspace_exists: bool = diesel::select(diesel::dsl::exists(
        workspaces_table::workspaces.filter(workspaces_data::id.eq(workspace_id)),
    ))
    .get_result(conn)?;
    if !workspace_exists {
        return Err(DieselError::NotFound);
    }

    let users = workspace_users_table::workspace_users
        .filter(workspace_users_data::workspace_id.eq(workspace_id))
        .inner_join(auth_users_table::auth_users)
        .select((
            auth_users_data::id,
            auth_users_data::username,
            auth_users_data::email,
        ))
        .load::<(i32, String, String)>(conn)?
        .into_iter()
        .map(|(id, username, email)| WorkspaceUserResponse {
            id,
            username,
            email,
        })
        .collect();

    Ok(users)
}
