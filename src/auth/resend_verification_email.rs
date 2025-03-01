use actix_web::{post, web::Json, HttpResponse};
use diesel::deserialize;
use serde::Deserialize;

use crate::{
    auth::confirmation_token::token::TokenType,
    models::{self, User},
    DPool,
};

use crate::auth::find_user::{Find, FindData};

use super::confirmation_token::token::{Cft, ConfirmationToken};

#[derive(Deserialize)]
struct ResendVerificationEmailRequest {
    email: String,
}

#[post("/resend_verification_email")]
pub async fn resend_verification_email(
    request: Json<ResendVerificationEmailRequest>,
    pool: DPool,
) -> HttpResponse {
    let user_data = FindData::find_by_email(request.email.clone(), pool.clone()).await;

    match user_data {
        Err(_) => HttpResponse::InternalServerError().json("Error while getting user data"),
        Ok(usr) => {
            match <Cft as ConfirmationToken>::send(
                usr.username,
                usr.email,
                pool,
                crate::auth::confirmation_token::token::TokenEmailType::AccountVerificationResend,
                None,
                true,
                TokenType::AccountVerification,
            )
            .await
            {
                Ok(_) => HttpResponse::Ok().json("Email resent successfully"),
                Err(_) => {
                    eprintln!("Error while resending verification email");
                    HttpResponse::InternalServerError().json("Server error while resending email")
                }
            }
        }
    }
}
