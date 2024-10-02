// only one active token shoukdad be

use actix_web::Responder;

use crate::{
    auth::{AuthError, VerificationTokenInvalid},
    DPool,
};

use crate::auth::confirmation_token::token::{Cft, ConfirmationToken};

pub async fn confirmation_token_test_helper(pool: DPool) -> impl Responder {
    let test_email: &str = "tomek@el-jot.eu";
    let c_token = Cft::new(test_email.to_string(), pool);

    match c_token {
        Ok(token) => {
            println!("confirmation_token: {}", token);
            actix_web::HttpResponse::Ok().json("token generated successfully")
        }
        Err(e) => {
            eprintln!("error with confirmation token, {:?}", e);
            actix_web::HttpResponse::Ok().json("error with generating token")
        }
    }
}

// it wont work it needs to pull token from somewhere and i dont know from where
pub async fn confirmation_token_verify_test_helper(token: String, pool: DPool) -> impl Responder {
    let confirmation = Cft::confirm(token, pool.clone());

    match confirmation {
        Ok(_) => actix_web::HttpResponse::Ok().json("Token confirmed successfully"),
        Err(e) => {
            eprintln!("Error verifying token {:?}", e);
            match e {
                AuthError::VerificationTokenError(invalid) => match invalid {
                    VerificationTokenInvalid::NotFound => {
                        actix_web::HttpResponse::BadRequest().json("Token not found")
                    }
                    VerificationTokenInvalid::Expired => {
                        actix_web::HttpResponse::BadRequest().json("Token has expired")
                    }
                    VerificationTokenInvalid::AccountAlreadyVerified => {
                        actix_web::HttpResponse::BadRequest().json("Account already verified")
                    }
                    VerificationTokenInvalid::ServerError => {
                        actix_web::HttpResponse::InternalServerError()
                            .json("Server error while verifying token")
                    }
                },
                AuthError::ServerError(message) => {
                    eprintln!("Server error: {}", message);
                    actix_web::HttpResponse::InternalServerError().json("Internal server error")
                }
                _ => {
                    eprintln!("Unknown error occurred: {:?}", e);
                    actix_web::HttpResponse::InternalServerError().json("Unknown error")
                }
            }
        }
    }
}
