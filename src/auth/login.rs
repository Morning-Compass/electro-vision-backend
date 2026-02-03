use crate::auth;
use crate::auth::confirmation_token::token;
use crate::auth::jwt;
use crate::auth::jwt::generate;
use crate::auth::ResponseUser;
use crate::auth::UserWithRoles;
use crate::constants::APPLICATION_JSON;
use crate::models::AuthUser as User;
use crate::response::Response as Res;
use crate::{est_conn, response, schema, DPool};
use actix_web::web;
use actix_web::{post, web::Json, HttpResponse};
use diesel::prelude::*;
use diesel::result::Error;
use serde::Deserialize;

type LoginUserError = response::Response<String>;
type LoginResponse = response::Response<ResponseUser>;
pub enum LoginMethodIdentifier {
    Username(String),
    Email(String),
    Token(String),
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

#[derive(Deserialize)]
pub struct RequestLoginToken {
    login_token: String,
}

pub async fn list_user(
    identifier: LoginMethodIdentifier,
    pool: DPool,
) -> Result<UserWithRoles, Error> {
    use crate::schema::auth_users::dsl::*;
    use crate::schema::confirmation_tokens::dsl::*;

    let user_result = match identifier {
        LoginMethodIdentifier::Username(user_username) => auth_users
            .filter(username.eq(&user_username))
            .first::<User>(&mut est_conn(pool.clone()))
            .optional()?,
        LoginMethodIdentifier::Email(_user_email) => auth_users
            .filter(email.eq(&_user_email))
            .first::<User>(&mut est_conn(pool.clone()))
            .optional()?,
        LoginMethodIdentifier::Token(user_token) => auth_users
            .inner_join(
                schema::confirmation_tokens::table
                    .on(schema::auth_users::email.eq(schema::confirmation_tokens::user_email)),
            )
            .filter(token.eq(user_token))
            .select(schema::auth_users::all_columns)
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

    let user_token = match generate(&usr.email) {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Error generating jwt");
            return Err(Error::NotFound);
        }
    };
    Ok(UserWithRoles::new(usr, user_roles_result, user_token))
}

#[post("/auth/login/username")]
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

#[post("/auth/login/email")]
pub async fn login_email(request: Json<RequestLoginEmail>, pool: DPool) -> HttpResponse {
    let user_email = request.email.clone();

    let user =
        web::block(move || list_user(LoginMethodIdentifier::Email(user_email), pool.clone()))
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

#[post("/auth/login/token")]
pub async fn login_token(request: Json<RequestLoginToken>, pool: DPool) -> HttpResponse {
    let req_token = request.login_token.clone();

    let valid_token = jwt::verify(&req_token, pool.clone());

    if !valid_token {
        return HttpResponse::Unauthorized().json(Res::new("Token invalid"));
    }

    let user = web::block(move || list_user(LoginMethodIdentifier::Token(req_token), pool.clone()))
        .await
        .unwrap();

    match user.await {
        Ok(usr) => HttpResponse::Ok()
            .content_type(APPLICATION_JSON)
            .json(LoginResponse::new(ResponseUser::new(usr))),
        Err(Error::NotFound) => {
            eprintln!("User with provided email was not found");
            HttpResponse::NotFound().json(LoginUserError::new("Token invalid".to_string()))
        }
        Err(e) => {
            eprintln!("Error matching users in login_username {:?}", e);
            HttpResponse::InternalServerError().json(LoginUserError {
                response: "Error ".to_string(),
            })
        }
    }
}
