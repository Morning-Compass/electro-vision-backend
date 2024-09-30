mod auth;
mod constants;
mod models;
mod response;
mod schema;
mod user;

use crate::auth::login::login::{login_email, login_username};
use crate::constants::CONNECTION_POOL_ERROR;
use actix_web::error::ErrorInternalServerError;
use actix_web::web::{self, Data};
use actix_web::{middleware, App, HttpResponse, HttpServer};
use auth::AuthError;
use chrono::Utc;
use diesel::result::Error;
use diesel::{
    r2d2::{self, ConnectionManager, Pool, PooledConnection},
    PgConnection,
};
use diesel::{ExpressionMethods, RunQueryDsl};
use dotenv::dotenv;
use models::User;
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

async fn insert_test_data(pool: DPool) -> Result<HttpResponse, actix_web::Error> {
    use crate::schema::users::dsl::*;

    // Hashing the password, map error to actix_web::Error
    let hashed_password = bcrypt::hash(constants::TEST_PASSWORD, bcrypt::DEFAULT_COST)
        .map_err(|_| ErrorInternalServerError("Failed to hash password"))?;

    // Insert data and handle Diesel errors, map them to Actix errors
    match diesel::insert_into(users)
        .values((
            username.eq(constants::TEST_USERNAME),
            email.eq(constants::TEST_EMAIL),
            password.eq(hashed_password),
            created_at.eq(Utc::now().naive_utc()),
            account_valid.eq(false),
        ))
        .get_result::<User>(&mut est_conn(pool))
    {
        Ok(_) => Ok(HttpResponse::Ok().json("test data inserted successfully")),
        Err(e) => {
            eprintln!("Error inserting user: {:?}", e);
            Err(ErrorInternalServerError("Failed to insert test data")) // Map Diesel error to HTTP 500
        }
    }
}

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
            .route("/insert_test_data", web::post().to(insert_test_data))
            .service(user::list)
            .service(auth::register::register)
            .service(login_email)
            .service(login_username)
            .service(auth::validate_account::validate_account)
    })
    .bind("127.0.0.1:3500")?
    .run()
    .await
}
