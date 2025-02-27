use actix_web::{post, put, web::Json, web::Path, HttpResponse};
use serde::Deserialize;

use crate::DPool;

pub struct EmailResetPasswordRequest {
    email: String,
}

pub struct ResetPasswordRequest {
    email: String,
}

#[derive(Deserialize)]
pub struct Token {
    token: String,
}

// endpoint to create password change token
#[put("/reset_password/{token}")]
pub fn email_reset_password(
    pool: DPool,
    request: Json<EmailResetPasswordRequest>,
    token: Path<Token>,
) -> HttpResponse {
    todo!();
}

//endpoint to send password
#[post("/reset_password")]
pub fn reset_password(pool: DPool, request: Json<ResetPasswordRequest>) -> HttpResponse {}
