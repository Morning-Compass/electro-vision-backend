use crate::constants::APPLICATION_JSON;
use crate::models::{self, User, UserRole};
use crate::{est_conn, response, schema, DPool};
use actix_web::web;
use actix_web::{post, web::Json, HttpResponse};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};

type LoginUser = response::Response<User>;
type LoginUserError = response::Response<String>;
type FullUser = response::Response<UserWithRoles>;

#[derive(Debug, Serialize, Deserialize)]
struct UserWithRoles {
    id: i32,
    username: String,
    password: String,
    email: String,
    created_at: NaiveDateTime,
    account_valid: bool,
    roles: Vec<String>,
}

struct ResponseUser {
    id: i32,
    username: String,
    email: String,
    created_at: NaiveDateTime,
    account_valid: bool,
    roles: Vec<String>,
}

impl UserWithRoles {
    fn new(user: User, roles: Vec<String>) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            password: user.password,
            created_at: user.created_at,
            account_valid: user.account_valid,
            roles,
        }
    }
}

impl ResponseUser {
    fn new(user: User, roles: Vec<String>) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            account_valid: user.account_valid,
            roles,
        }
    }
}

#[derive(Deserialize)]
pub struct RequestLoginUsername {
    username: String,
    password: String,
}

pub async fn list_user_by_username(
    user_username: String,
    pool: DPool,
) -> Result<UserWithRoles, Error> {
    use crate::schema::roles::dsl::*;
    use crate::schema::user_roles::dsl::*;
    use crate::schema::users::dsl::*;

    let user_result = users
        .filter(username.eq(&user_username))
        .first::<User>(&mut est_conn(pool.clone()))
        .optional()?;

    let usr = match user_result {
        Some(user) => user,
        None => return Err(Error::NotFound),
    };

    let user_roles_result = schema::user_roles::table
        .inner_join(schema::roles::table)
        .filter(schema::user_roles::user_id.eq(usr.id))
        .select(schema::roles::name)
        .load::<String>(&mut est_conn(pool))
        .unwrap_or_else(|_| vec![]);

    Ok(UserWithRoles::new(usr, user_roles_result))
}

#[post("/login-username")]
pub async fn login_username(request: Json<RequestLoginUsername>, pool: DPool) -> HttpResponse {
    let user_username = request.username.clone();
    let user = web::block(move || list_user_by_username(user_username, pool))
        .await
        .unwrap();

    match user.await {
        Ok(usr) => match bcrypt::verify(&request.password, &usr.password) {
            Ok(valid) if valid => HttpResponse::Ok()
                .content_type(APPLICATION_JSON)
                .json(FullUser::new(usr)),
            Ok(_) => HttpResponse::BadRequest()
                .json(LoginUserError::new("password is incorrect".to_string())),
            Err(_) => {
                eprintln!(
                    "given password: \n {} \n db password \n {}",
                    &request.password, &usr.password
                );
                HttpResponse::InternalServerError()
                    .json(LoginUserError::new("Failed to verify password".to_string()))
            }
        },
        Err(Error::NotFound) => {
            eprintln!("User with provided username was not found");
            HttpResponse::NotFound().json(LoginUserError::new(format!(
                "User with username {} was not found",
                request.username,
            )))
        }
        Err(e) => {
            eprintln!("Error matching users in login_username {:?}", e);
            HttpResponse::InternalServerError().json(LoginUserError {
                response: "Error ".to_string(),
            })
        }
    }
}
