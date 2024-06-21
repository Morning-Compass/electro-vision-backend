mod db_models;
mod db_utils;
mod schema;

use actix::SyncArbiter;
use actix_web::{get, post, web, web::Data, App, HttpResponse, HttpServer, Responder};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use dotenv::dotenv;

use db_utils::{get_pool, AppState, DbActor};
use std::env;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool: Pool<ConnectionManager<PgConnection>> = get_pool(&db_url);
    let db_addr = SyncArbiter::start(5, move || DbActor(pool.clone()));
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                db: db_addr.clone(),
            }))
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
