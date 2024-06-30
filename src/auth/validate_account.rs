use crate::auth::confirmation_token::token::{Cft, ConfirmationToken, ConfirmationTokenRequest};
use crate::constants::APPLICATION_JSON;
use crate::{response, DPool};
use actix_web::put;
use actix_web::web::Path;
use actix_web::{web::Json, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct ValidateAccountRequest {
    email: String,
}

#[derive(Deserialize)]
pub struct Token {
    token: String,
}

type ValidateResponse = response::Response<String>;

#[put("/validate/{token}")]
pub async fn validate_account(
    request: Json<ValidateAccountRequest>,
    user_token: Path<Token>,
    pool: DPool,
) -> HttpResponse {
    println!("{}", user_token.token);
    match <Cft as ConfirmationToken>::confirm(
        ConfirmationTokenRequest {
            user_email: request.email.clone(),
            token_str: user_token.token.clone(),
        },
        pool,
    ) {
        Ok(_) => HttpResponse::Ok()
            .content_type(APPLICATION_JSON)
            .json(ValidateResponse::new(
                "Account verified successfully".to_string(),
            )),
        Err(e) => match e {
            diesel::result::Error::NotFound => HttpResponse::BadRequest()
                .json(ValidateResponse::new("Token not found".to_string())),
            diesel::result::Error::DatabaseError(_, info) => {
                eprintln!("Error validating account {:?}", info);
                HttpResponse::InternalServerError().json(ValidateResponse::new(
                    "Error processing account validation".to_string(),
                ))
            }
            _ => HttpResponse::InternalServerError().json(ValidateResponse::new(
                "An unexpected error occurred".to_string(),
            )),
        },
    }
}
