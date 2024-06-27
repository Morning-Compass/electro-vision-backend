use crate::models::User;
use crate::schema::users;
use crate::user::NoIdUser;
use crate::{est_conn, response, DPool};
use actix_web::web;
use actix_web::{post, web::Json, HttpResponse};
use diesel::prelude::*;
use serde_derive::Deserialize;

#[derive(Deserialize, Clone)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

type Register = response::Response<String>;

pub async fn insert_user(new_user: NoIdUser, pool: DPool) -> Result<Register, String> {
    match diesel::insert_into(users::table)
        .values((
            users::username.eq(new_user.username),
            users::email.eq(new_user.email),
            users::password.eq(new_user.password),
            users::created_at.eq_all(new_user.created_at),
            users::account_valid.eq(new_user.account_valid),
        ))
        .execute(&mut est_conn(pool))
    {
        Ok(_) => Ok(Register::new("User registered successfully".to_string())),
        Err(e) => {
            eprintln!("Error inserting new user: {:?}", e);
            Err("Error registering user".to_string())
        }
    }
}

#[post("/register")]
pub async fn register(request: Json<RegisterRequest>, pool: DPool) -> HttpResponse {
    let new_user = User::new(
        request.username.clone(),
        request.email.clone(),
        request.password.clone(),
    );

    let registered_user = web::block(move || insert_user(new_user, pool))
        .await
        .map_err(|e| {
            eprintln!("Failed to list users: {:?}", e);
            HttpResponse::InternalServerError().finish()
        })
        .unwrap();

    match registered_user.await {
        Ok(_) => HttpResponse::Ok().json(Register {
            response: "User registered successfully".to_string(),
        }),
        Err(e) => {
            eprintln!("Error registering user {:?}", e);
            HttpResponse::InternalServerError().json(Register {
                response: "Error registering user".to_string(),
            })
        }
    }
}
