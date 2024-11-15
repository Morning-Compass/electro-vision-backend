use actix_web::{post, web::Json, HttpResponse};
use diesel::deserialize;
use serde::Deserialize;

use crate::{models::User, DPool};

use super::{confirmation_token::token::ConfirmationToken, find_user::Find};

#[derive(Deserialize)]
struct ResendVerificationEmailRequest {
    email: String,
}

#[post("/resend_verification_email")]
pub async fn resend_verification_email(
    request: Json<ResendVerificationEmailRequest>,
    pool: DPool,
) -> HttpResponse {
    match Find::find_by_email(request.email, pool.clone()).await {
        Ok(usr) => {
            match ConfirmationToken::send(
                usr.username,
                usr.email,
                pool,
                crate::auth::confirmation_token::token::TokenEmailType::AccountVerificationResend,
                None,
                true,
            ) {
                Ok(_) => HttpResponse::Ok().json("Email resend successfully"),
                Err(e) => {
                    eprintln!("Error while resending verification email: {:?}", e);
                    HttpResponse::InternalServerError().json("Server error while resending email")
                }
            }
        }
        Err(_) => {
            eprintln!("Error while checking for user by email");
            return HttpResponse::InternalServerError().json("Server error");
        }
    }
}
