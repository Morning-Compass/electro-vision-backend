use crate::response::Response as Res;
use crate::schema::auth_users::dsl as auth_users_table;
use crate::schema::ev_subscriptions::dsl as ev_subscriptions_table;
use crate::schema::workspaces::dsl as workspaces_table;
use crate::{schema::auth_users as auth_users_data, DBPConn};

use actix_web::{post, web::Json, HttpResponse};
use chrono::NaiveDateTime;
use diesel::{result::Error as DieselError, ExpressionMethods};
use diesel::{JoinOnDsl, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};

use crate::{est_conn, DPool};

#[derive(Deserialize)]
struct ListWorkspacesRequest {
    email: String,
}

#[derive(Serialize)]
struct WorkspaceResponse {
    id: i32,
    plan_file_name: String,
    start_date: NaiveDateTime,
    finish_date: Option<NaiveDateTime>,
    geolocation: Option<String>,
    ev_subscription: String,
    name: String,
    role: String, //
}

#[post("/workspace/list")]
pub async fn list_workspaces(pool: DPool, req: Json<ListWorkspacesRequest>) -> HttpResponse {
    let email = req.email.clone();
    let conn = &mut est_conn(pool);

    match get_workspaces(conn, email).await {
        Ok(workspaces) => HttpResponse::Ok().json(Res::new(workspaces)),
        Err(err) => {
            eprintln!("Error listing workspaces: {}", err);
            HttpResponse::InternalServerError()
                .json(Res::new("Server error while listing workspaces"))
        }
    }
}

async fn get_workspaces(
    conn: &mut DBPConn,
    owner_email: String,
) -> Result<Vec<WorkspaceResponse>, DieselError> {
    use crate::schema::{
        auth_users::dsl as au, ev_subscriptions::dsl as evs, workspace_roles::dsl as wr,
        workspace_users::dsl as wu, workspaces::dsl as ws,
    };

    // Step 1: Find user by email to get their ID
    let user = au::auth_users
        .filter(au::email.eq(owner_email.clone()))
        .first::<crate::models::AuthUser>(conn)?;

    // Step 2: Join workspaces the user is a part of (via workspace_users)
    let results = ws::workspaces
        .inner_join(wu::workspace_users.on(wu::workspace_id.eq(ws::id)))
        .inner_join(evs::ev_subscriptions.on(ws::ev_subscription_id.eq(evs::id)))
        .inner_join(wr::workspace_roles.on(wr::id.eq(wu::workspace_role_id)))
        .filter(wu::user_id.eq(user.id))
        .select((
            ws::id,
            ws::plan_file_name,
            ws::start_date,
            ws::finish_date,
            ws::geolocation,
            evs::subscription,
            ws::name,
            wr::name, // the user's role in this workspace
        ))
        .load::<(
            i32,
            String,
            NaiveDateTime,
            Option<NaiveDateTime>,
            Option<String>,
            String,
            String,
            String, // role name
        )>(conn)?;

    let workspaces = results
        .into_iter()
        .map(
            |(
                id,
                plan_file_name,
                start_date,
                finish_date,
                geolocation,
                ev_subscription,
                name,
                role,
            )| {
                WorkspaceResponse {
                    id,
                    plan_file_name,
                    start_date,
                    finish_date,
                    geolocation,
                    ev_subscription,
                    name,
                    role, // include this in the response struct
                }
            },
        )
        .collect();

    Ok(workspaces)
}
