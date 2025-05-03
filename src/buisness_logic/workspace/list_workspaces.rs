use crate::response::Response as Res;
use crate::schema::auth_users::dsl as auth_users_table;
use crate::schema::workspaces::dsl as workspaces_table;
use crate::{schema::auth_users as auth_users_data, DBPConn};

use actix_web::{
    post,
    web::Json,
    HttpResponse,
};
use diesel::{result::Error as DieselError, ExpressionMethods};
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use serde::Deserialize;

use crate::{est_conn, models, DPool};

#[derive(Deserialize)]
struct ListWorkspacesRequest {
    email: String,
}

#[post("/list/workspace")]
pub async fn list_workspaces(pool: DPool, req: Json<ListWorkspacesRequest>) -> HttpResponse {
    let _email = req.email.clone();

    let conn = &mut est_conn(pool);

    match get_workspaces(conn, _email).await {
        Ok(w) => HttpResponse::Ok().json(Res::new(w)),
        Err(err) => {
            eprintln!("Error listing workspace users: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Server error while listing users"))
        }
    }
}

async fn get_workspaces(
    conn: &mut DBPConn,
    owner_email: String,
) -> Result<Vec<models::Workspace>, DieselError> {
    let workspaces = workspaces_table::workspaces
        .inner_join(auth_users_table::auth_users)
        .filter(auth_users_data::email.eq(owner_email))
        .select(models::Workspace::as_select())
        .load::<models::Workspace>(conn)?;

    Ok(workspaces)
}
