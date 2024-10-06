#[cfg(test)]
mod tests {

    use diesel::{
        r2d2::{self, ConnectionManager},
        PgConnection,
    };
    use dotenv::dotenv;

    use crate::{
        auth::{
            self,
            find_user::{Find, FindData},
        },
        DBPool, DPool,
    };
    use serde_json::json;
    use std::env;

    use actix_web::{test, web, App};
    use std::str;

    pub async fn login_with_roles_helper_email(pool: DPool) -> impl actix_web::Responder {
        let user_email = "tomek@el-jot.eu";

        let user_data = auth::login::login::list_user(
            auth::login::login::LoginMethodIdentifier::Email(user_email.to_string()),
            pool.clone(),
        );
        match user_data.await {
            Ok(user) => actix_web::HttpResponse::Ok().json(json!({
                "message": "user",
                "user": user
            })),
            Err(e) => actix_web::HttpResponse::InternalServerError().json(json!({
                "error": "error while listing user by email",
                "details": e.to_string()
            })),
        }
    }

    pub async fn login_with_roles_helper_username(pool: DPool) -> impl actix_web::Responder {
        let username = "tomek";
        let user_data = auth::login::login::list_user(
            auth::login::login::LoginMethodIdentifier::Username(username.to_string()),
            pool.clone(),
        );

        match user_data.await {
            Ok(user) => {
                println!("User data: {:?}", user);
                actix_web::HttpResponse::Ok().json(json!({
                    "message": "user",
                    "user": user,
                }))
            }
            Err(e) => actix_web::HttpResponse::InternalServerError().json(json!({
                "error": "error while listing user by email",
                "details": e.to_string(),
            })),
        }
    }

    pub async fn change_password_helper(pool: DPool) -> impl actix_web::Responder {
        let email: &str = "tomek@el-jot.eu";

        let user_data = FindData::find_by_email(email.to_string(), pool).await;
        match user_data {
            Ok(user) => {
                println!("\n USER DATA: \n{:?}", user);
                actix_web::HttpResponse::Ok().json(json!({
                    "message": "found the user",
                    "user": user,
                }))
            }
            Err(e) => actix_web::HttpResponse::InternalServerError().json(json!({
                "error": "error while changing password",
                "details": e.to_string(),
            })),
        }
    }

    fn setup_pool() -> DBPool {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to pool")
    }

    #[actix_web::test]
    async fn test_login_with_roles_email() {
        let pool = setup_pool();
        let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).route(
            "/login_by_email",
            web::get().to(login_with_roles_helper_email),
        ))
        .await;

        let req = test::TestRequest::get().uri("/login_by_email").to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success(), "Listing user by email failed");

        let body = test::read_body(resp).await;
        let body_str = str::from_utf8(&body).expect("Failed to convert body to string");
        assert!(
            body_str.contains(r#""message":"user""#),
            "Unexpected response body: {:?}",
            body_str
        );
    }

    #[actix_web::test]
    async fn test_login_with_roles_username() {
        let pool = setup_pool();
        let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).route(
            "/login_by_username",
            web::get().to(login_with_roles_helper_username),
        ))
        .await;

        let req = test::TestRequest::get()
            .uri("/login_by_username")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(
            resp.status().is_success(),
            "Listing user by username failed"
        );

        let body = test::read_body(resp).await;
        let body_str = str::from_utf8(&body).expect("Failed to convert body to string");
        assert!(
            body_str.contains(r#""message":"user""#),
            "Unexpected response body: {:?}",
            body_str
        );
    }

    #[actix_web::test]
    async fn test_change_password() {
        let pool = setup_pool();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/change_password", web::put().to(change_password_helper)),
        )
        .await;

        let req = test::TestRequest::put()
            .uri("/change_password")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success(), "Password change request failed");

        let body = test::read_body(resp).await;
        let body_str = str::from_utf8(&body).expect("Failed to convert body to string");
        assert!(
            body_str.contains(r#""message":"found the user""#),
            "Unexpected response body: {:?}",
            body_str
        );
    }
}
