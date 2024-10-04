use crate::{
    auth::{self, jwt::jwt_decode}, constants, DPool
};
use core::panic;

const TEST_EMAIL: &str = constants::TEST_EMAIL;

pub fn generate_token() -> String {
    match auth::jwt::generate(&TEST_EMAIL) {
        Ok(token) => {
            println!("token: {}", token);
            token
        }
        Err(e) => panic!("Error generating token, {}", e),
    }
}

pub fn decode() {
    match jwt_decode(generate_token()) {
        Ok(claims) => {
            println!("Decoded claims: {:?}", claims.claims);
        }
        Err(e) => {
            panic!("Failed to decode token: {:?}", e);
        }
    }
}

pub async fn helper_validate_token(pool: DPool) -> impl actix_web::Responder {
    match auth::jwt::verify(&generate_token(), pool) {
        true => actix_web::HttpResponse::Ok().json("Logged with token successfully".to_string()),
        false => {
            actix_web::HttpResponse::BadRequest().json("Error logging in with token".to_string())
        }
    }
}
