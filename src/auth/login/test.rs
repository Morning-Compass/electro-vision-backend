#[cfg(test)]
mod tests {
    use std::env;
    use actix::dev::Request;
    use actix_web::web::{route, Path};
    use diesel::r2d2::{self, ConnectionManager};
    use diesel::PgConnection;
    use dotenv::dotenv;
    use serde_json::json;

    use crate::auth::find_user::{Find, FindData};
    use crate::{auth, DPool};

    async fn login_with_roles_helper_email(pool: DPool) -> impl actix_web::Responder {
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

    async fn login_with_roles_helper_username(pool: DPool) -> impl actix_web::Responder {
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
            },
            Err(e) => actix_web::HttpResponse::InternalServerError().json(json!({
                "error": "error while listing user by email",
                "details": e.to_string(),
            }))
        }
    }
    async fn change_password(pool: DPool) -> impl actix_web::Responder {
        let email: &str = "tomek@el-jot.eu";

        let is_found = FindData::find_by_email(email.to_string(), pool).await;
        match is_found {
            Ok(found) => {
                if found {
                    actix_web::HttpResponse::Ok().json(json!({
                        "message": "found the user",
                        "user": found,
                    }))
                }
                else {
                    actix_web::HttpResponse::Ok().json(json!({
                        "message": "Did not find the user",
                        "user": found,
                    }))
                }

            },
            Err(e) => actix_web::HttpResponse::InternalServerError().json(json!({
                "error": "error while changing password",
                "details": e.to_string(),
            })),
        }
    }
    #[actix_web::test]
    async fn login_with_roles() {
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
                    "/login_with_roles_email",
                    actix_web::web::get().to(login_with_roles_helper_email),
                )
                .route(
                    "/login_with_roles_username",
                    actix_web::web::get().to(login_with_roles_helper_username),
                )
                .route(
                    "/change-password",
                    actix_web::web::get().to(change_password)
                ),
        )
        .await;

        let req = actix_web::test::TestRequest::get()
            .uri("/change-password")
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert!(resp.status().is_success())
    }
}
