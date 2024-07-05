mod auth;
mod constants;
mod models;
mod response;
mod schema;
mod user;

use std::env;
use std::fs::File;
use std::io::Read;

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

pub type ResponseKeys = Data<serde_json::Value>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let mut file = File::open("api-response.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let response_keys = response::JsonResponse::read(&contents);

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(pool.clone()))
            .app_data(actix_web::web::Data::new(response_keys.clone()))
            .wrap(middleware::Logger::default())
            .service(user::list)
            .service(auth::register::register)
            .service(auth::login::login_username)
            .service(auth::login::login_email)
            .service(auth::validate_account::validate_account)
    })
    .bind("127.0.0.1:3500")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_est_conn() -> PooledConnection<ConnectionManager<PgConnection>> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool");
        pool.get().expect(CONNECTION_POOL_ERROR)
    }

    #[test]
    fn jwt_generate_and_decode() {

        // Assuming you have a valid token for testing
        let token = match auth::jwt::generate("tomek@el-jot.eu") {
            Ok(token) => {
                println!("token: {}", token);
                token
            }
            Err(e) => panic!("Error generating token: {}", e),
        };

        match auth::jwt::verify(&*token, test_est_conn()) {
            true => println!("token verified"),
            false => println!("token not verified"),
        }
    }
}
