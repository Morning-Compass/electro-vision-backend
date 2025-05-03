use actix_web::{
    post,
    web::{Json, Path},
    HttpResponse,
};
use serde::Deserialize;

use crate::{est_conn, DPool};

#[derive(Deserialize)]
struct ListWorkspaceUsersRequest {
    email: String,
}

#[derive(Deserialize)]
struct ListWorkspaceUsersPath {
    id: String,
}

#[post("/list/workspace/{id}")]
pub async fn list_workspace_users(
    pool: DPool,
    req: Json<ListWorkspaceUsersRequest>,
    path: Path<ListWorkspaceUsersPath>,
) -> HttpResponse {
    let email = req.email.clone();
    let workspace_id = path.id.clone();

    let conn = &mut est_conn(pool);

    todo!("Implement list_workspace_users")
}
