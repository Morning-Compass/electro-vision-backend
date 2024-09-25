#[cfg(test)]
// only one active token shoukdad be
mod test {

    use actix_web::Responder;
    use diesel::{
        r2d2::{self, ConnectionManager},
        PgConnection,
    };
    use dotenv::dotenv;

    use crate::{
        auth::{AuthError, VerificationTokenInvalid},
        DPool,
    };
    use std::env;

    use crate::auth::confirmation_token::token::{Cft, ConfirmationToken};

    async fn confirmation_token_test_helper(pool: DPool) -> impl Responder {
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

    async fn confirmation_token_verify_test_helper(token: String, pool: DPool) -> impl Responder {
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

    #[actix_web::test]
    async fn confirmation_token_test() {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool");

        let app = actix_web::test::init_service(
            actix_web::App::new()
                .app_data(actix_web::web::Data::new(pool.clone()))
                .route(
                    "/create_confirmation_token",
                    actix_web::web::get().to(confirmation_token_test_helper),
                )
                .route(
                    "/confirm_token",
                    actix_web::web::get().to(confirmation_token_verify_test_helper),
                ),
        )
        .await;

        let req = actix_web::test::TestRequest::get()
            .uri("/login_token")
            .uri("/confirm_token")
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert!(resp.status().is_success())
    }
}
