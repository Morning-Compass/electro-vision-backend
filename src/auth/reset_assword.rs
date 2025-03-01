use actix_web::{post, put, web::Json, web::Path, HttpResponse};
use bcrypt::bcrypt;
use diesel::query_dsl::methods::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};
use serde::Deserialize;

use crate::auth::auth_error::AccountVerification;
use crate::est_conn;
use crate::schema::users as user_data;
use crate::schema::users::dsl as user_table;

use crate::{
    auth::{
        confirmation_token::token::{Cft, TokenType},
        VerificationTokenError,
    },
    constants::APPLICATION_JSON,
    DPool,
};

use super::confirmation_token::token::ConfirmationToken;

#[derive(Deserialize)] // Add Deserialize
pub struct EmailResetPasswordRequest {
    new_password: String,
    email: String,
}

#[derive(Deserialize)] // Add Deserialize
pub struct ResetPasswordRequest {
    email: String,
}

#[derive(Deserialize)]
pub struct Token {
    token: String,
}

async fn change_password(email: String, password: String, pool: DPool) -> Result<(), ()> {
    let hashed_password = match bcrypt::hash(password, bcrypt::DEFAULT_COST) {
        Ok(hp) => hp,
        Err(_) => return Err(()),
    };
    match diesel::update(user_table::users.filter(user_data::email.eq(email)))
        .set(user_data::password.eq(hashed_password))
        .execute(&mut est_conn(pool))
    {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

#[put("/reset_password/{token}")]
pub async fn email_reset_password(
    pool: DPool,
    req: Json<EmailResetPasswordRequest>,
    token: Path<Token>,
) -> HttpResponse {
    let pool_clone: DPool = pool.clone();
    match <Cft as ConfirmationToken>::confirm(
        token.token.clone(),
        TokenType::PasswordReset(req.email.clone()),
        pool_clone,
    ) {
        Ok(_) => match change_password(req.email.clone(), req.new_password.clone(), pool).await {
            Ok(_) => HttpResponse::Ok()
                .content_type(APPLICATION_JSON)
                .json(String::from("Password verificated successfully")),
            Err(_) => {
                HttpResponse::InternalServerError().json("An unexpected error occured".to_string())
            }
        },

        Err(e) => match e {
            VerificationTokenError::NotFound => HttpResponse::BadRequest().json("Token not found"),
            VerificationTokenError::Expired => HttpResponse::BadRequest().json("Token expired"),
            VerificationTokenError::Account(AccountVerification::AccountAlreadyVerified) => {
                HttpResponse::BadRequest().json("Account already veryfied")
            }
            VerificationTokenError::TokenAlreadyExists => {
                HttpResponse::BadRequest().json("Token already exists")
            }
            VerificationTokenError::ServerError(_) => {
                HttpResponse::InternalServerError().json("An unexpected error occured".to_string())
            }
        },
    }
}

#[post("/reset_password")]
pub async fn reset_password(pool: DPool, request: Json<ResetPasswordRequest>) -> HttpResponse {
    match <Cft as ConfirmationToken>::new(
        request.email.clone(),
        false,
        TokenType::PasswordReset(request.email.clone()),
        pool,
    ) {
        Ok(_) => HttpResponse::Ok().json("Email send with verification link"),
        Err(e) => match e {
            VerificationTokenError::NotFound => HttpResponse::BadRequest().json("Token not found"),
            VerificationTokenError::Expired => HttpResponse::BadRequest().json("Token expired"),
            VerificationTokenError::TokenAlreadyExists => {
                HttpResponse::BadRequest().json("Token already exists")
            }
            VerificationTokenError::ServerError(_) => {
                HttpResponse::InternalServerError().json("An unexpected error occured".to_string())
            }
            // any other error can not ocurr due to password not being account
            _ => {
                HttpResponse::InternalServerError().json("An unexpected error occured".to_string())
            }
        },
    }
}
