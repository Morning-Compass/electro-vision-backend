use crate::constants::APPLICATION_JSON;
use crate::models::User;
use crate::{est_conn, response, schema, DPool};
use actix_web::web;
use actix_web::{post, web::Json, HttpResponse};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};

type LoginUserError = response::Response<String>;
type LoginResponse = response::Response<ResponseUser>;
pub enum LoginMethodIdentifier {
    Username(String),
    Email(String),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithRoles {
    id: i32,
    username: String,
    password: String,
    email: String,
    created_at: NaiveDateTime,
    account_valid: bool,
    roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseUser {
    id: i32,
    username: String,
    email: String,
    created_at: NaiveDateTime,
    account_valid: bool,
    roles: Vec<String>,
}

impl UserWithRoles {
    fn new(user: User, roles: Vec<String>) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            password: user.password,
            created_at: user.created_at,
            account_valid: user.account_valid,
            roles,
        }
    }
}

impl ResponseUser {
    fn new(user: UserWithRoles) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            account_valid: user.account_valid,
            roles: user.roles,
        }
    }
}

#[derive(Deserialize)]
pub struct RequestLoginUsername {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RequestLoginEmail {
    email: String,
    password: String,
}

pub async fn list_user(identifier: LoginMethodIdentifier, pool: DPool) -> Result<UserWithRoles, Error> {
    use crate::schema::users::dsl::*;

    let user_result = match identifier {
        LoginMethodIdentifier::Username(user_username) => users
            .filter(username.eq(&user_username))
            .first::<User>(&mut est_conn(pool.clone()))
            .optional()?,
        LoginMethodIdentifier::Email(user_email) => users
            .filter(email.eq(&user_email))
            .first::<User>(&mut est_conn(pool.clone()))
            .optional()?,
    };

    let usr = match user_result {
        Some(user) => user,
        None => return Err(Error::NotFound),
    };

    let user_roles_result = schema::user_roles::table
        .inner_join(schema::roles::table)
        .filter(schema::user_roles::user_id.eq(usr.id))
        .select(schema::roles::name)
        .load::<String>(&mut est_conn(pool))
        .unwrap_or_else(|_| vec![]);

    Ok(UserWithRoles::new(usr, user_roles_result))
}

#[post("/login-username")]
pub async fn login_username(request: Json<RequestLoginUsername>, pool: DPool) -> HttpResponse {
    let user_username = request.username.clone();
    let user = web::block(move || list_user(LoginMethodIdentifier::Username(user_username), pool))
        .await
        .unwrap();

    match user.await {
        Ok(usr) => match bcrypt::verify(&request.password, &usr.password) {
            Ok(valid) if valid => HttpResponse::Ok()
                .content_type(APPLICATION_JSON)
                .json(LoginResponse::new(ResponseUser::new(usr))),
            Ok(_) => HttpResponse::BadRequest()
                .json(LoginUserError::new("password is incorrect".to_string())),
            Err(_) => HttpResponse::InternalServerError()
                .json(LoginUserError::new("Failed to verify password".to_string())),
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

#[post("/login-email")]
pub async fn login_email(request: Json<RequestLoginEmail>, pool: DPool) -> HttpResponse {
    let user_email = request.email.clone();
    let user = web::block(move || list_user(LoginMethodIdentifier::Email(user_email), pool))
        .await
        .unwrap();

    match user.await {
        Ok(usr) => match bcrypt::verify(&request.password, &usr.password) {
            Ok(valid) if valid => HttpResponse::Ok()
                .content_type(APPLICATION_JSON)
                .json(LoginResponse::new(ResponseUser::new(usr))),
            Ok(_) => HttpResponse::BadRequest()
                .json(LoginUserError::new("password is incorrect".to_string())),
            Err(_) => HttpResponse::InternalServerError()
                .json(LoginUserError::new("Failed to verify password".to_string())),
        },
        Err(Error::NotFound) => {
            eprintln!("User with provided email was not found");
            HttpResponse::NotFound().json(LoginUserError::new(format!(
                "User with email {} was not found",
                request.email,
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

