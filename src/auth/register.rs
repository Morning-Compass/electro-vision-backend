use crate::models::User;
use crate::user::NoIdUser;
use crate::{est_conn, response, DPool};
use actix_web::web;
use actix_web::{post, web::Json, HttpResponse};
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error;
use serde_derive::Deserialize;

#[derive(Deserialize, Clone)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

type Register = response::Response<String>;

pub async fn insert_user(new_user: NoIdUser, pool: DPool) -> Result<Register, Error> {
    use crate::schema::users::dsl::*;

    let hashed_passowrd = match bcrypt::hash(new_user.password, bcrypt::DEFAULT_COST) {
        Ok(hp) => hp,
        Err(_) => return Err(diesel::result::Error::RollbackTransaction),
    };

    diesel::insert_into(users)
        .values((
            username.eq(new_user.username),
            email.eq(new_user.email),
            password.eq(hashed_passowrd),
            created_at.eq_all(new_user.created_at),
            account_valid.eq(new_user.account_valid),
        ))
        .execute(&mut est_conn(pool))
        .map(|_| Register::new("User registered successfully".to_string()))
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
        .unwrap();

    match registered_user.await {
        Ok(_) => HttpResponse::Ok().json(Register {
            response: "User registered successfully!".to_string(),
        }),
        Err(e) => match e {
            Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                if let Some(existing_email) = info.details() {
                    eprintln!("Email already exists: {}", existing_email);
                }
                HttpResponse::BadRequest().json(Register {
                    response: "Email already exists".to_string(),
                })
            }
            _ => {
                eprintln!("Error registering user: {:?}", e);
                HttpResponse::InternalServerError().json(Register {
                    response: "Error registering user".to_string(),
                })
            }
        },
    }
}
