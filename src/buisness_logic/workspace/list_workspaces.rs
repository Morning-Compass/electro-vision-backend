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
    let results = workspaces_table::workspaces
        .inner_join(
            auth_users_table::auth_users.on(workspaces_table::owner_id.eq(auth_users_table::id)),
        )
        .inner_join(
            ev_subscriptions_table::ev_subscriptions
                .on(workspaces_table::ev_subscription_id.eq(ev_subscriptions_table::id)),
        )
        .filter(auth_users_data::email.eq(owner_email))
        .select((
            workspaces_table::id,
            workspaces_table::plan_file_name,
            workspaces_table::start_date,
            workspaces_table::finish_date,
            workspaces_table::geolocation,
            ev_subscriptions_table::subscription,
            workspaces_table::name,
        ))
        .load::<(
            i32,
            String,
            NaiveDateTime,
            Option<NaiveDateTime>,
            Option<String>,
            String,
            String,
        )>(conn)?;

    let workspaces = results
        .into_iter()
        .map(
            |(id, plan_file_name, start_date, finish_date, geolocation, ev_subscription, name)| {
                WorkspaceResponse {
                    id,
                    plan_file_name,
                    start_date,
                    finish_date,
                    geolocation,
                    ev_subscription,
                    name,
                }
            },
        )
        .collect();

    Ok(workspaces)
}
