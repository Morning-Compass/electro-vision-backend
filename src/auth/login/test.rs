use crate::auth::find_user::{Find, FindData};
use crate::{auth, DPool};

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

pub async fn change_password(pool: DPool) -> impl actix_web::Responder {
    let email: &str = "tomek@el-jot.eu";

    let is_found = FindData::find_by_email(email.to_string(), pool).await;
    match is_found {
        Ok(found) => {
            if found {
                actix_web::HttpResponse::Ok().json(json!({
                    "message": "found the user",
                    "user": found,
                }))
            } else {
                actix_web::HttpResponse::Ok().json(json!({
                    "message": "Did not find the user",
                    "user": found,
                }))
            }
        }
        Err(e) => actix_web::HttpResponse::InternalServerError().json(json!({
            "error": "error while changing password",
            "details": e.to_string(),
        })),
    }
}
