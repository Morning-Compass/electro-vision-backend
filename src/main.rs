
mod endpoints;
mod models;
mod schema;
mod services;

#[macro_use]
extern crate diesel;

use actix_web::{dev::ServiceRequest, web, App, Error, HttpServer};
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
    PgConnection,
};
use dotenv::dotenv;

use std::env;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let db_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&db_url).unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let db_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool: Pool = Pool::builder()
        .build(ConnectionManager::<PgConnection>::new(db_url))
        .expect("Failed to create pool");

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .service(endpoints::get_users)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
