use crate::models::User;
use crate::schema::users;
use crate::{est_conn, response, DPool};
use actix_web::web;
use actix_web::{post, web::Json, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use serde_derive::Deserialize;

#[derive(Deserialize, Clone)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

type Register = response::Response<String>;

pub async fn insert_user(new_user: User, pool: DPool) -> Result<Register, String> {
    match diesel::insert_into(users::table)
        .values(&new_user)
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
    let new_user = User {
        id: 0,
        username: request.username.clone(),
        email: request.email.clone(),
        password: request.password.clone(),
        created_at: Utc::now().naive_utc(),
        account_valid: false,
    };

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
