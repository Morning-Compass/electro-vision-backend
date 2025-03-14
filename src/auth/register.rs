use actix_web::web;
use actix_web::{post, web::Json, HttpResponse};
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error;
use serde_derive::Deserialize;

use auth::confirmation_token::token::ConfirmationToken;

use crate::auth::confirmation_token::token::Cft;
use crate::models::User;
use crate::response::JsonResponse;
use crate::user::NoIdUser;
use crate::{auth, est_conn, DPool, ResponseKeys};

#[derive(Deserialize, Clone)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

pub async fn insert_user(new_user: NoIdUser, pool: DPool) -> Result<User, Error> {
    use crate::schema::users::dsl::*;

    let hashed_password = match bcrypt::hash(new_user.password, bcrypt::DEFAULT_COST) {
        Ok(hp) => hp,
        Err(_) => return Err(diesel::result::Error::RollbackTransaction),
    };

    match diesel::insert_into(users)
        .values((
            username.eq(new_user.username),
            email.eq(new_user.email),
            password.eq(hashed_password),
            created_at.eq_all(new_user.created_at),
            account_valid.eq(new_user.account_valid),
        ))
        .get_result::<User>(&mut est_conn(pool))
    {
        Ok(usr) => Ok(usr),
        Err(e) => {
            eprintln!("Error inserting user {:?}", e);
            Err(e)
        }
    }
}

pub async fn insert_user_roles(usr_id: i32, pool: DPool) -> Result<String, Error> {
    use crate::schema::roles::dsl::{name as role_name, roles};
    use crate::schema::user_roles::dsl::*;
    let mut conn = est_conn(pool);

    let role_id_value: i16 = roles
        .filter(role_name.eq("USER"))
        .select(crate::schema::roles::dsl::id)
        .first::<i16>(&mut conn)?;

    match diesel::insert_into(user_roles)
        .values((user_id.eq(usr_id), role_id.eq(role_id_value)))
        .execute(&mut conn)
    {
        Ok(_) => Ok("User role assigned successfully".to_string()),
        Err(e) => {
            eprintln!("Error inserting user_roles: {:?}", e);
            Err(e)
        }
    }
}

pub struct RegisterJsonResponse {
    ok_key: String,
    ok_value: String,
    err_internal_key: String,
    err_internal_value: String,
    err_email_exists_key: String,
    err_email_exists_value: String,
}

#[post("/register")]
pub async fn register(
    request: Json<RegisterRequest>,
    pool: DPool,
    response_keys: ResponseKeys,
) -> HttpResponse {
    let keys = RegisterJsonResponse {
        ok_key: response_keys["register_success"]["key"].to_string(),
        ok_value: response_keys["register_success"]["message"].to_string(),
        err_internal_key: response_keys["register_server_error"]["key"].to_string(),
        err_internal_value: response_keys["register_server_error"]["message"].to_string(),
        err_email_exists_key: response_keys["register_client_error"]["key"].to_string(),
        err_email_exists_value: response_keys["register_client_error"]["message"].to_string(),
    };

    let new_user = User::new(
        request.username.clone(),
        request.email.clone(),
        request.password.clone(),
    );
    let pool_clone = pool.clone();

    let registered_user = web::block(move || insert_user(new_user, pool_clone))
        .await
        .unwrap();

    match registered_user.await {
        Ok(usr) => {
            match (
                insert_user_roles(usr.id, pool.clone()).await,
                <Cft as ConfirmationToken>::new(request.email.clone(), pool),
            ) {
                (Ok(_), Ok(_)) => {
                    HttpResponse::Ok().json(JsonResponse::new(keys.ok_key, keys.ok_value))
                }
                (Err(e), _) => {
                    eprintln!("Error inserting user roles: {:?}", e);
                    HttpResponse::InternalServerError().json(JsonResponse::new(
                        keys.err_internal_key,
                        keys.err_internal_value,
                    ))
                }
                (_, Err(e)) => {
                    eprintln!("Error while creating token: {:?}", e);
                    HttpResponse::InternalServerError().json(JsonResponse::new(
                        keys.err_internal_key,
                        keys.err_internal_value,
                    ))
                }
            }
        }
        Err(e) => match e {
            Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                if let Some(existing_email) = info.details() {
                    eprintln!("Email already exists: {}", existing_email);
                }
                HttpResponse::BadRequest().json(JsonResponse::new(
                    keys.err_email_exists_key,
                    keys.err_email_exists_value,
                ))
            }
            _ => {
                eprintln!("Error registering user: {:?}", e);
                HttpResponse::InternalServerError().json(JsonResponse::new(
                    keys.err_internal_key,
                    keys.err_internal_value,
                ))
            }
        },
    }
}
