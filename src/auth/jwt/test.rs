#[cfg(test)]
mod tests {
    use diesel::{
        r2d2::{self, ConnectionManager},
        PgConnection,
    };
    use dotenv::dotenv;

    use crate::{
        auth::{self, jwt::jwt_decode},
        DPool,
    };
    use core::panic;
    use std::env;

    const TEST_EMAIL: &str = "tomek@el-jot.eu";

    fn generate_token() -> String {
        match auth::jwt::generate(&TEST_EMAIL) {
            Ok(token) => {
                println!("token: {}", token);
                token
            }
            Err(e) => panic!("Error generating token, {}", e),
        }
    }

    #[test]
    fn decode() {
        match jwt_decode(generate_token()) {
            Ok(claims) => {
                println!("Decoded claims: {:?}", claims.claims);
            }
            Err(e) => {
                panic!("Failed to decode token: {:?}", e);
            }
        }
    }

    async fn helper_validate_token(pool: DPool) -> impl actix_web::Responder {
        match auth::jwt::verify(&generate_token(), pool) {
            true => {
                actix_web::HttpResponse::Ok().json("Logged with token successfully".to_string())
            }
            false => actix_web::HttpResponse::BadRequest()
                .json("Error logging in with token".to_string()),
        }
    }

    #[actix_web::test]
    async fn validate_token() {
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
                    "/login_token",
                    actix_web::web::get().to(helper_validate_token),
                ),
        )
        .await;

        let req = actix_web::test::TestRequest::get()
            .uri("/login_token")
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert!(resp.status().is_success())
    }
}
