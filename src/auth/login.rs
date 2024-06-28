use crate::constants::APPLICATION_JSON;
use crate::models::User;
use crate::{est_conn, response, DPool};
use actix_web::web;
use actix_web::{post, web::Json, HttpResponse};
use diesel::prelude::*;
use diesel::result::Error;
use serde::Deserialize;

type LoginUser = response::Response<User>;
type LoginUserError = response::Response<String>;

#[derive(Deserialize)]
struct RequestLoginUsername {
    username: String,
    password: String,
}

pub async fn list_user_by_username(
    req: RequestLoginUsername,
    pool: DPool,
) -> Result<LoginUser, Error> {
    use crate::schema::users::dsl::*;
    let hashed_password = match bcrypt::hash(req.password, bcrypt::DEFAULT_COST) {
        Ok(hp) => hp,
        Err(_) => return Err(Error::RollbackTransaction),
    };

    match users
        .filter(username.eq(req.username))
        .filter(password.eq(hashed_password))
        .first::<User>(&mut est_conn(pool))
    {
        Ok(usr) => Ok(LoginUser::new(usr)),
        Err(e) => {
            eprintln!("Error selecting user, {:?}", e);
            Err(Error::RollbackTransaction)
        }
    }
}

#[post("/login-username")]
pub async fn login_username(request: Json<RequestLoginUsername>, pool: DPool) -> HttpResponse {
    let user_username_credentials = RequestLoginUsername {
        username: request.username.clone(),
        password: request.password.clone(),
    };
    let user = web::block(move || list_user_by_username(user_username_credentials, pool))
        .await
        .unwrap();

    match user.await {
        Ok(usr) => HttpResponse::Ok()
            .content_type(APPLICATION_JSON)
            .json(LoginUser {
                response: usr.response,
            }),
        Err(e) => {
            eprintln!("Error matching users in login_username {:?}", e);
            HttpResponse::BadRequest().json(LoginUserError {
                response: "Error ".to_string(),
            })
        }
    }
}
