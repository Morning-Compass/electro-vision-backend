use actix_web::HttpResponse;
use actix_web::put;
use actix_web::web::Path;
use serde::Deserialize;

use crate::{DPool, response};
use crate::auth::confirmation_token::token::{Cft, ConfirmationToken};
use crate::constants::APPLICATION_JSON;

#[derive(Deserialize)]
pub struct Token {
    token: String,
}

type ValidateResponse = response::Response<String>;

#[put("/validate/{token}")]
pub async fn validate_account(user_token: Path<Token>, pool: DPool) -> HttpResponse {
    println!("{}", user_token.token);
    match <Cft as ConfirmationToken>::confirm(user_token.token.clone(), pool) {
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
            diesel::result::Error::AlreadyInTransaction => {
                HttpResponse::BadRequest()
                    .json(ValidateResponse::new("Account already verified".to_string()))
            },
            _ => HttpResponse::InternalServerError().json(ValidateResponse::new(
                "An unexpected error occurred".to_string(),
            )),
        },
    }
}
