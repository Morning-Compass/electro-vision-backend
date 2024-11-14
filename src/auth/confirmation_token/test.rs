#[cfg(test)]
mod tests {
    use actix_web::{test, web, App, Responder};
    use diesel::{
        r2d2::{self, ConnectionManager},
        PgConnection,
    };
    use dotenv::dotenv;
    use std::env;
    use std::str;

    use crate::{
        auth::{
            confirmation_token::token::{Cft, ConfirmationToken},
            AuthError, VerificationTokenError,
        },
        DBPool, DPool,
    };

    /// Helper function to generate a confirmation token
    pub async fn confirmation_token_test_helper(pool: DPool) -> impl Responder {
        let test_email: &str = "tomek@el-jot.eu";
        let c_token = Cft::new(test_email.to_string(), pool);

        match c_token {
            Ok(token) => {
                println!("----------   OK   ----------");
                println!("confirmation_token: {}", token);
                actix_web::HttpResponse::Ok().json("token generated successfully")
            }
            Err(e) => {
                println!("----------   ERROR   ----------");
                eprintln!("error with confirmation token, {:?}", e);
                actix_web::HttpResponse::Ok().json("error with generating token")
            }
        }
    }

    /// Helper function to verify the confirmation token
    pub async fn confirmation_token_verify_test_helper(
        token: String,
        pool: DPool,
    ) -> impl Responder {
        let confirmation = Cft::confirm(token, pool.clone());

        match confirmation {
            Ok(_) => {
                println!("----------   OK   ----------");
                actix_web::HttpResponse::Ok().json("Token confirmed successfully")
            }
            Err(e) => {
                println!("----------   ERROR   ----------");
                eprintln!("Error verifying token {:?}", e);
                match e {
                    VerificationTokenError::NotFound => {
                        actix_web::HttpResponse::BadRequest().json("Token not found")
                    }
                    VerificationTokenError::Expired => {
                        actix_web::HttpResponse::BadRequest().json("Token has expired")
                    }
                    VerificationTokenError::AccountAlreadyVerified => {
                        actix_web::HttpResponse::BadRequest().json("Account already verified")
                    }
                    VerificationTokenError::ServerError(_) => {
                        actix_web::HttpResponse::InternalServerError()
                            .json("Server error while verifying token")
                    }
                }
            }
        }
    }

    /// Setup database pool
    fn setup_pool() -> DBPool {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to pool")
    }

    /// Test for generating a confirmation token
    #[actix_web::test]
    async fn test_generate_confirmation_token() {
        let pool = setup_pool();
        let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).route(
            "/generate_token",
            web::get().to(confirmation_token_test_helper),
        ))
        .await;

        let req = test::TestRequest::get().uri("/generate_token").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(
            resp.status().is_success(),
            "Token generation request failed"
        );

        let body = test::read_body(resp).await;
        let body_str = str::from_utf8(&body).expect("Failed to convert body to string");
        assert!(
            body_str.contains("token generated successfully"),
            "Unexpected response body: {:?}",
            body_str
        );
    }

    /// Test for verifying a confirmation token
    #[actix_web::test]
    async fn test_verify_confirmation_token() {
        let pool = setup_pool();
        let token = "some_test_token".to_string(); // Replace with a valid test token

        let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).route(
            "/verify_token",
            web::get().to(confirmation_token_verify_test_helper),
        ))
        .await;

        let req = test::TestRequest::get().uri("/verify_token").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(
            resp.status().is_success(),
            "Token verification request failed"
        );

        let body = test::read_body(resp).await;
        let body_str = str::from_utf8(&body).expect("Failed to convert body to string");

        // This will depend on the token status (valid/expired/verified), adapt as needed
        assert!(
            body_str.contains("Token confirmed successfully")
                || body_str.contains("Token not found")
                || body_str.contains("Token has expired")
                || body_str.contains("Account already verified"),
            "Unexpected response body: {:?}",
            body_str
        );
    }
}
