use actix_web::put;
use actix_web::web::Path;
use actix_web::HttpResponse;
use serde::Deserialize;

use crate::auth::confirmation_token::token::{Cft, ConfirmationToken};
use crate::constants::APPLICATION_JSON;
use crate::{response, DPool};

#[derive(Deserialize)]
pub struct Token {
    token: String,
}

type ValidateResponse = response::Response<String>;

#[put("/validate/{token}")]
pub async fn validate_account(user_token: Path<Token>, pool: DPool) -> HttpResponse {
    use crate::auth::{AuthError, VerificationTokenInvalid};
    println!("{}", user_token.token);
    match <Cft as ConfirmationToken>::confirm(user_token.token.clone(), pool) {
        Ok(_) => HttpResponse::Ok()
            .content_type(APPLICATION_JSON)
            .json(ValidateResponse::new(
                "Account verified successfully".to_string(),
            )),
        Err(e) => match e {
            AuthError::VerificationTokenError(VerificationTokenInvalid::NotFound) => {
                HttpResponse::BadRequest()
                    .json(ValidateResponse::new("Token not found".to_string()))
            }
            AuthError::ServerError(e) => {
                eprintln!("Error while account verification: {}", e);
                HttpResponse::InternalServerError().json("Error verificating account")
            }
            AuthError::VerificationTokenError(VerificationTokenInvalid::AccountAlreadyVerified) => {
                HttpResponse::BadRequest().json(ValidateResponse::new(
                    "Account already verified".to_string(),
                ))
            }
            _ => HttpResponse::InternalServerError().json(ValidateResponse::new(
                "An unexpected error occurred".to_string(),
            )),
        },
    }
}
