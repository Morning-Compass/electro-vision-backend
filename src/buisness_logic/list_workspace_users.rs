use crate::{est_conn, DPool};

#[derive(Deserialize)]
struct ListWorkspaceUsersRequest {
    email: String,
}

#[post("/list/workspace/{id}")]
pub async fn list_workspace_users(
    pool: DPool,
    req: Json<ListWorkspaceUsersRequest>,
) -> HttpResponse {
    todo!("Implement list_workspace_users")
}
