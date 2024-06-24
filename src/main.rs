mod db_utils;
mod models;
mod schema;
mod services;

use actix::SyncArbiter;
use actix_web::{web::Data, App, HttpServer};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use dotenv::dotenv;

use db_utils::{get_pool, AppState, DbActor};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool: Pool<ConnectionManager<PgConnection>> = get_pool(&db_url);
    let db_addr = SyncArbiter::start(5, move || DbActor(pool.clone()));
    HttpServer::new(move || {
        App::new().app_data(Data::new(AppState {
            db: db_addr.clone(),
        }))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
