use super::establish_connection;
use super::models::*;
use super::schema::users::dsl::*;
use crate::diesel::{
    dsl::{delete, insert_into},
    prelude::*,
    QueryDsl, RunQueryDsl,
};
use actix_web::{get, web, Error, HttpResponse};
use serde::{Deserialize, Serialize};

pub struct InputUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[get("/users")]
pub async fn get_users() -> Result<HttpResponse, Error> {
    let conn = &mut establish_connection();
    Ok(web::block(move || {
        let db_users = users
            .limit(5)
            .select(username)
            .load(conn)
            .expect("Error loading users");
    })
    .await
    .map(|user| HttpResponse::Ok().json(user))
        .map_err(|_| HttpResponse::InternalServerError().json("Working not")?)
}
