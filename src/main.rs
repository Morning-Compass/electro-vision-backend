mod auth;
mod constants;
mod models;
mod response;
mod schema;
mod user;

use std::env;

use crate::constants::CONNECTION_POOL_ERROR;
use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use diesel::{
    r2d2::{self, ConnectionManager, Pool, PooledConnection},
    PgConnection,
};
use dotenv::dotenv;

type DBPool = Pool<ConnectionManager<PgConnection>>;
pub type DBPConn = PooledConnection<ConnectionManager<PgConnection>>;
pub type DPool = Data<DBPool>;
pub fn est_conn(pool: Data<DBPool>) -> PooledConnection<ConnectionManager<PgConnection>> {
    pool.get().expect(CONNECTION_POOL_ERROR)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(user::list)
            .service(auth::register::register)
            .service(auth::login::login_username)
            .service(auth::login::login_email)
    })
    .bind("127.0.0.1:3500")?
    .run()
    .await
}
