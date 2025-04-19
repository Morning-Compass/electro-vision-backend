use crate::auth::auth_error::AccountVerification;
use crate::auth::confirmation_token::token::TokenEmailType;
use crate::auth::find_user::Find;
use crate::est_conn;
use crate::models::AuthUser as User;
use crate::response::Response as Res;
use crate::schema::auth_users as user_data;
use crate::schema::auth_users::dsl as user_table;
use actix_web::{post, put, web::Json, web::Path, HttpResponse};
use diesel::query_dsl::methods::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};
use serde::Deserialize;

use crate::{
    auth::{
        confirmation_token::token::{Cft, TokenType},
        VerificationTokenError,
    },
    constants::APPLICATION_JSON,
    DPool,
};

use super::confirmation_token::token::ConfirmationToken;
use super::find_user::FindData;

#[derive(Deserialize)] // Add Deserialize
struct EmailResetPasswordRequest {
    new_password: String,
    email: String,
}

#[derive(Deserialize)] // Add Deserialize
struct ResetPasswordRequest {
    email: String,
}

#[derive(Deserialize)]
struct Token {
    token: String,
}

async fn change_password(email: String, password: String, pool: DPool) -> Result<(), ()> {
    let hashed_password = match bcrypt::hash(password, bcrypt::DEFAULT_COST) {
        Ok(hp) => hp,
        Err(_) => return Err(()),
    };
    match diesel::update(user_table::auth_users.filter(user_data::email.eq(email)))
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
                .json(Res::new("Password verified successfully")),
            Err(_) => {
                HttpResponse::InternalServerError().json(Res::new("An unexpected error occurred"))
            }
        },

        Err(e) => match e {
            VerificationTokenError::NotFound => {
                HttpResponse::BadRequest().json(Res::new("Token not found"))
            }
            VerificationTokenError::Expired => {
                HttpResponse::BadRequest().json(Res::new("Token expired"))
            }
            VerificationTokenError::Account(AccountVerification::AccountAlreadyVerified) => {
                HttpResponse::BadRequest().json(Res::new("Account already verified"))
            }
            VerificationTokenError::TokenAlreadyExists => {
                HttpResponse::BadRequest().json(Res::new("Token already exists"))
            }
            VerificationTokenError::ServerError(_) => {
                HttpResponse::InternalServerError().json(Res::new("An unexpected error occurred"))
            }
        },
    }
}

#[post("/reset_password")]
pub async fn reset_password(pool: DPool, request: Json<ResetPasswordRequest>) -> HttpResponse {
    let pool_clone = pool.clone();
    let user: User =
        match <FindData as Find>::find_by_email(request.email.clone(), pool_clone).await {
            Ok(u) => u,
            Err(_) => return HttpResponse::InternalServerError().json(Res::new("Unknown error")),
        };

    match <Cft as ConfirmationToken>::send(
        user.username,
        request.email.clone(),
        pool.clone(),
        TokenEmailType::PasswordReset,
        None,
        false,
        TokenType::PasswordReset(request.email.clone()),
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().json(Res::new("Email send with verification link")),
        Err(e) => {
            match e {
                VerificationTokenError::NotFound => {
                    HttpResponse::BadRequest().json(Res::new("Token not found"))
                }
                VerificationTokenError::Expired => {
                    HttpResponse::BadRequest().json(Res::new("Token expired"))
                }
                VerificationTokenError::TokenAlreadyExists => {
                    HttpResponse::BadRequest().json(Res::new("Token already exists"))
                }
                VerificationTokenError::ServerError(_) => HttpResponse::InternalServerError()
                    .json(Res::new("An unexpected error occured")),
                // any other error can not ocurr due to password not being account
                _ => HttpResponse::InternalServerError()
                    .json(Res::new("An unexpected error occured")),
            }
        }
    }
}
