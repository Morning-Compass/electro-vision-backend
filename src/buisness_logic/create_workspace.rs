use crate::auth::find_user::FindData;
use crate::response::Response as Res;
use crate::schema::workspaces::dsl as workspaces_table;
use crate::{auth::find_user::Find, schema::workspaces as workspaces_data};
use actix_web::{post, web::Json, HttpResponse};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use diesel::prelude::Insertable;
use diesel::{Connection, ExpressionMethods, RunQueryDsl};
use serde::Deserialize;

use crate::{est_conn, models, schema, DPool};

#[derive(Deserialize)]
struct CreateWorkspaceRequest {
    owner_email: String,
    name: String,
    finish_date: Option<NaiveDate>,
    plan_file_name: Option<String>,
    geolocation: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::workspaces)]
struct NewWorkspace<'a> {
    owner_id: i32,
    geolocation: Option<&'a str>,
    plan_file_name: Option<&'a str>,
    start_date: NaiveDateTime,
    finish_date: Option<NaiveDate>,
    ev_subscription_id: i32,
    name: String,
}

#[post("/create_workspace")]
pub async fn create_workspace(pool: DPool, req: Json<CreateWorkspaceRequest>) -> HttpResponse {
    let user = match <FindData as Find>::find_by_email(req.owner_email.clone(), pool.clone()).await
    {
        Ok(user) => user,
        Err(_) => {
            eprintln!("User not found for workspace creations");
            return HttpResponse::InternalServerError()
                .json(Res::new("Server error while creating workspace"));
        }
    };

    let conn = &mut est_conn(pool);

    let result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        let new_workspace = NewWorkspace {
            owner_id: user.id,
            geolocation: req.geolocation.as_deref(),
            plan_file_name: req.plan_file_name.as_deref(),
            start_date: Utc::now().naive_utc(),
            finish_date: req.finish_date,
            ev_subscription_id: 1,
            name: req.name.clone(),
        };
        let workspace = diesel::insert_into(workspaces_table::workspaces)
            .values(&new_workspace)
            .execute(conn);

        Ok(workspace)
    });

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("Workspace created successfully")),
        Err(err) => {
            eprintln!("Error creating workspace: {}", err);
            HttpResponse::InternalServerError()
                .json(Res::new("Server error while creating workspace"))
        }
    }
}
