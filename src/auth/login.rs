use crate::constants::APPLICATION_JSON;
use crate::models::User;
use crate::{est_conn, response, DPool};
use actix_web::web;
use actix_web::{post, web::Json, HttpResponse};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};

type LoginUser = response::Response<User>;
type LoginUserSuccess = response::Response<ResponseUser>;
type LoginUserError = response::Response<String>;

#[derive(Serialize)]
struct ResponseUser {
    id: i32,
    username: String,
    email: String,
    created_at: NaiveDateTime,
    account_valid: bool,
}

impl ResponseUser {
    fn new(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            account_valid: user.account_valid,
        }
    }
}

#[derive(Deserialize)]
pub struct RequestLoginUsername {
    username: String,
    password: String,
}

pub async fn list_user_by_username(user_username: String, pool: DPool) -> Result<LoginUser, Error> {
    use crate::schema::users::dsl::*;

    users
        .filter(username.eq(user_username))
        .first::<User>(&mut est_conn(pool))
        .map(|usr| LoginUser::new(usr))
}

#[post("/login-username")]
pub async fn login_username(request: Json<RequestLoginUsername>, pool: DPool) -> HttpResponse {
    let user_username = request.username.clone();
    let user = web::block(move || list_user_by_username(user_username, pool))
        .await
        .unwrap();

    match user.await {
        Ok(usr) => match bcrypt::verify(&request.password, &usr.response.password) {
            Ok(valid) if valid => HttpResponse::Ok()
                .content_type(APPLICATION_JSON)
                .json(LoginUserSuccess::new(ResponseUser::new(usr.response))),
            Ok(_) => HttpResponse::BadRequest()
                .json(LoginUserError::new("password is incorrect".to_string())),
            Err(_) => {
                eprintln!(
                    "given password: \n {} \n db password \n {}",
                    &request.password, &usr.response.password
                );
                HttpResponse::InternalServerError()
                    .json(LoginUserError::new("Failed to verify password".to_string()))
            }
        },
        Err(Error::NotFound) => {
            eprintln!("User with provided username was not found");
            HttpResponse::NotFound().json(LoginUserError::new(format!(
                "User with username {} was not found",
                request.username,
            )))
        }
        Err(e) => {
            eprintln!("Error matching users in login_username {:?}", e);
            HttpResponse::InternalServerError().json(LoginUserError {
                response: "Error ".to_string(),
            })
        }
    }
}
