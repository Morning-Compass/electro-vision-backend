#[cfg(test)]
mod tests {

    use diesel::{
        r2d2::{self, ConnectionManager},
        PgConnection,
    };
    use dotenv::dotenv;
    use serde::Serialize;

    use crate::{
        auth::{
            self,
            find_user::{Find, FindData},
            hash_password::Hash,
        },
        models::User,
        DBPool, DPool,
    };
    use serde_json::json;
    use std::{borrow::Borrow, env};

    use crate::user::UserChangePassword;
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

    pub async fn change_password_helper(
        pool: DPool,
        data: web::Json<UserChangePassword>,
    ) -> impl actix_web::Responder {
        println!(
            "Data - password: {:?}, email: {:?}",
            data.password, data.email
        );

        let user_data = FindData::find_by_email(data.email.to_string(), pool).await;
        match user_data {
            Ok(mut user) => {
                user.password = data.password.to_string();
                actix_web::HttpResponse::Ok().json(json!({
                    "message": "user password changed",
                    "user": user,
                }))
            }
            Err(e) => actix_web::HttpResponse::InternalServerError().json(json!({
                "error": "error while changing user password",
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
        use crate::auth::hash_password::HashPassword;
        let req_data = UserChangePassword {
            email: "tomek@el-jot.eu".to_string(),
            password: HashPassword::hash_password("123".to_string()).await,
        };

        let pool = setup_pool();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/change-password", web::put().to(change_password_helper)),
        )
        .await;

        let req = test::TestRequest::put()
            .uri("/change-password")
            .set_json(req_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success(), "Password change request failed");

        let body = test::read_body(resp).await;
        println!("code: 23123123123123131231\n{:?}\n", body);

        let body_str = str::from_utf8(&body).expect("Failed to convert body to string");
        assert!(
            body_str.contains(r#""user password changed""#),
            "Unexpected response body: {:?}",
            body_str
        );
    }
}
