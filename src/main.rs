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
    const TEST_EMAIL: &str = "tomek@el-jot.eu";

    fn generate_token() -> String {
        match auth::jwt::generate(&TEST_EMAIL) {
            Ok(token) => {
                println!("token: {}", token);
                return token;
            }
            Err(e) => panic!("Error generating token: {}", e),
        };
    }

    async fn helper_validate_token(pool: DPool) -> impl actix_web::Responder {
        match auth::jwt::verify(&generate_token(), pool) {
            true => {
                actix_web::HttpResponse::Ok().json("Logged with token successfully".to_string())
            }
            false => actix_web::HttpResponse::BadRequest()
                .json("Error logging in with token".to_string()),
        }
    }

    #[actix_web::test]
    async fn validate_token() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool");

        let app = actix_web::test::init_service(
            actix_web::App::new()
                .app_data(actix_web::web::Data::new(pool.clone()))
                .route(
                    "/login_token",
                    actix_web::web::get().to(helper_validate_token),
                ),
        )
        .await;

        let req = actix_web::test::TestRequest::get()
            .uri("/login_token")
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert!(resp.status().is_success())
    }
}
