use crate::auth::find_user::{Find, FindData};
use crate::auth::hash_password::Hash;
use crate::models::User;
use crate::response::Response;
use crate::response_handler::{ResponseError, ResponseHandler, ResponseTrait};
use crate::schema::users;
use crate::{constants::APPLICATION_JSON, models};
use actix_web::{
    get, put,
    web::Json,
    web::{self},
    HttpResponse,
};
use chrono::{NaiveDateTime, Utc};
use diesel::deserialize;
use diesel::{prelude::*, result::Error};
use serde::{Deserialize, Serialize};

use crate::{est_conn, DPool};

pub type Users = Response<Vec<User>>;

pub struct NoIdUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub account_valid: bool,
}

#[derive(Deserialize)]
pub struct UserEmail {
    pub email: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserChangePassword {
    pub email: String,
    pub password: String,
}

impl models::User {
    pub fn new(username: String, email: String, password: String) -> NoIdUser {
        NoIdUser {
            username,
            email,
            password,
            created_at: Utc::now().naive_utc(),
            account_valid: false,
        }
    }
}

pub async fn list_users(amount: i64, pool: DPool) -> Result<Users, Error> {
    use crate::schema::users::dsl::*;

    let users_query = users
        .select(User::as_select())
        .order(created_at.desc())
        .limit(amount)
        .load::<User>(&mut est_conn(pool))
        .unwrap_or_else(|e| {
            eprintln!("Error querying users {:?}", e);
            vec![]
        });

    Ok(Users {
        response: users_query.into_iter().collect(),
    })
}

#[get("/users")]
pub async fn list(pool: DPool) -> HttpResponse {
    let users_listed = web::block(move || list_users(50, pool))
        .await
        .map_err(|e| {
            eprintln!("Failed to list users: {:?}", e);
            HttpResponse::InternalServerError().finish()
        })
        .unwrap();

    match users_listed.await {
        Ok(users) => HttpResponse::Ok()
            .content_type(APPLICATION_JSON)
            .json(users),
        Err(e) => {
            eprintln!("Failed to serialize users {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[put("/change-password")]
pub async fn change_password(request: Json<UserChangePassword>, pool: DPool) -> HttpResponse {
    use crate::auth::hash_password::HashPassword;
    use crate::schema::users::dsl::*;

    let contents = match ResponseHandler::file_get_contents("./api-response.json".to_string()).await
    {
        Ok(content) => content,
        Err(e) => {
            println!("\n\n\n\n ERROR {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("An error occurred: {:#?}", e)
            }));
        }
    };

    // println!("Content of file: {:?}", contents);

    let conn = &mut est_conn(pool);

    let new_password = HashPassword::hash_password(request.password.to_string()).await;

    let update_result = diesel::update(users.filter(email.eq(request.email.to_string())))
        .set(password.eq(new_password))
        .execute(conn);

    match update_result {
        Ok(_) => HttpResponse::Ok().json("user password changed"),
        Err(e) => {
            HttpResponse::InternalServerError().json(("Change password error: {}", e.to_string()))
        }
    };

    HttpResponse::Ok().json("user password changed")
}
