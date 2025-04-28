use crate::response::Response as Res;
use actix_web::put;
use actix_web::web::Path;
use actix_web::HttpResponse;
use serde::Deserialize;

use crate::auth::auth_error::{AccountVerification, VerificationTokenError};
use crate::auth::confirmation_token::token::{Cft, ConfirmationToken, TokenType};
use crate::constants::APPLICATION_JSON;
use crate::DPool;

#[derive(Deserialize)]
pub struct Token {
    token: String,
}

#[put("/validate/{token}")]
pub async fn validate_account(user_token: Path<Token>, pool: DPool) -> HttpResponse {
    println!("{}", user_token.token);
    match <Cft as ConfirmationToken>::confirm(
        user_token.token.clone(),
        TokenType::AccountVerification,
        pool,
    )
    .await
    {
        Ok(_) => HttpResponse::Ok()
            .content_type(APPLICATION_JSON)
            .json(Res::new("Account verified successfully".to_string())),
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
            VerificationTokenError::ServerError(_) => HttpResponse::InternalServerError()
                .json(Res::new("An unexpected error occurred".to_string())),
            _ => HttpResponse::InternalServerError()
                .json(Res::new("An unexpected error occurred".to_string())),
        },
    }
}
