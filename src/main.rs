mod auth;
mod buisness_logic;
mod constants;
mod emails;
mod models;
mod models_insertable;
mod response;
mod response_handler;
mod schema;
mod user;

use crate::auth::login::login::{login_email, login_username};
use crate::constants::CONNECTION_POOL_ERROR;
use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use chrono::Utc;
use constants::DOMAIN;
use diesel::{
    r2d2::{self, ConnectionManager, Pool, PooledConnection},
    PgConnection,
};
use dotenv::dotenv;
use std::env;
use std::fs::File;
use std::io::Read;

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

    println!("{}", Utc::now().naive_utc());

    let mut file = File::open("api-response.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let response_keys = response::JsonResponse::read(&contents);

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    println!("now: {}", Utc::now().naive_utc());

    HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(actix_web::web::Data::new(pool.clone()))
            .app_data(actix_web::web::Data::new(response_keys.clone()))
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .service(user::list)
            .service(auth::register::register)
            .service(login_email)
            .service(login_username)
            .service(auth::validate_account::validate_account)
            .service(auth::resend_verification_email::resend_verification_email)
            .service(auth::reset_password::reset_password)
            .service(auth::reset_password::email_reset_password)
            .service(auth::verify_session::verify_session)
            .service(buisness_logic::workspace::add_user_to_workspace::add_user_to_workspace)
            .service(buisness_logic::workspace::workspace_invitation::workspace_invitation)
            .service(buisness_logic::workspace::list_workspace_users::list_workspace_users)
            .service(buisness_logic::workspace::create_workspace::create_workspace)
            .service(buisness_logic::workspace::create_task::create_task)
            .service(buisness_logic::workspace::list_tasks::list_tasks)
            .service(buisness_logic::workspace::list_workspaces::list_workspaces)
            .service(buisness_logic::full_user::read::get_full_user)
            .service(buisness_logic::full_user::register::register_full_user)
    })
    .bind(DOMAIN)?
    .run()
    .await
}
