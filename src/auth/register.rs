use actix_web::web;
use actix_web::{post, web::Json, HttpResponse};
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error;
use serde_derive::Deserialize;

use auth::confirmation_token::token::ConfirmationToken;

use crate::auth::auth_error::AccountVerification;
use crate::auth::confirmation_token::token::{Cft, TokenType};
use crate::auth::jwt::generate;
use crate::auth::{ResponseUser, UserWithRoles};
use crate::models::User;
use crate::response::JsonResponse;
use crate::response::Response as Res;
use crate::user::NoIdUser;
use crate::{auth, est_conn, schema, DPool, ResponseKeys};

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
    use crate::schema::roles::dsl::{id, name, roles};
    use crate::schema::user_roles::dsl::*;
    let mut conn = est_conn(pool);

    println!("uid {}", usr_id);

    let role_id_value = roles
        .filter(name.eq("USER"))
        .select(id)
        .first::<i32>(&mut conn)?;

    match diesel::insert_into(user_roles)
        //.values((user_id.eq(usr_id), role_id.eq(role_id_value)))
        .values((user_id.eq(usr_id), role_id.eq(role_id_value)))
        .execute(&mut conn)
    {
        Ok(_) => Ok("User role assigned successfully".to_string()),
        Err(e) => {
            eprintln!(
                "Error inserting user_roles while registration okay bruv : {:?}",
                e
            );
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

pub struct OkResponse {
    message: String,
    user: ResponseUser,
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
            let usrclone = usr.clone();
            let email_clone = usr.email.clone();
            let username_clone = usr.username.clone();
            match (
                insert_user_roles(usrclone.id, pool.clone()).await,
                <Cft as ConfirmationToken>::new(
                    request.email.clone(),
                    false,
                    TokenType::AccountVerification,
                    pool.clone(),
                ),
            ) {
                (Ok(_), Ok(tok)) => {
                    use crate::auth::auth_error::VerificationTokenError;
                    match <Cft as ConfirmationToken>::send(
                        username_clone, // `username` is moved here
                        email_clone,    // Use the cloned `email`
                        pool.clone(),
                        auth::confirmation_token::token::TokenEmailType::AccountVerification,
                        Some(tok),
                        false,
                        TokenType::AccountVerification,
                    )
                    .await
                    {
                        Ok(_) => {
                            let email = usr.email.clone();
                            let token = match generate(&email) {
                                Ok(t) => t,
                                Err(_) => {
                                    eprintln!("Error generating jwt");
                                    return HttpResponse::InternalServerError()
                                        .json(Res::new("Error generating jwt"));
                                }
                            };

                            let user_roles_result = schema::user_roles::table
                                .inner_join(schema::roles::table)
                                .filter(schema::user_roles::user_id.eq(usr.id))
                                .select(schema::roles::name)
                                .load::<String>(&mut est_conn(pool))
                                .unwrap_or_else(|_| vec![]);
                            HttpResponse::Ok().json(Res::new(ResponseUser::new(
                                UserWithRoles::new(usr, user_roles_result, token),
                            )))
                        }
                        Err(e) => match e {
                            VerificationTokenError::NotFound => HttpResponse::BadRequest()
                                .json(Res::new("Token invalid or not generated yet")),
                            VerificationTokenError::Account(
                                AccountVerification::AccountAlreadyVerified,
                            ) => HttpResponse::BadRequest()
                                .json(Res::new("Account has already been verified".to_string())),
                            VerificationTokenError::Expired => HttpResponse::BadRequest()
                                .json(Res::new("Token has expired".to_string())),
                            VerificationTokenError::ServerError(_) => {
                                eprintln!("{:?}", e);
                                HttpResponse::InternalServerError()
                                    .json(Res::new("Server error while veryfing account"))
                            }
                            VerificationTokenError::TokenAlreadyExists => {
                                HttpResponse::BadRequest()
                                    .json(Res::new("Verification token already exists"))
                            }
                        },
                    }
                }
                (Err(e), _) => {
                    eprintln!(
                        "Error inserting user roles while registration brumv: {:?}",
                        e
                    );
                    HttpResponse::InternalServerError()
                        .json(Res::new("Something went wrong during registration"))
                }
                (_, Err(e)) => {
                    eprintln!("Error while creating token: {:?}", e);
                    HttpResponse::InternalServerError()
                        .json(Res::new("Something went wrong during token creation"))
                }
            }
        }
        Err(e) => match e {
            Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                if let Some(existing_email) = info.details() {
                    eprintln!("Email already exists: {}", existing_email);
                }
                HttpResponse::BadRequest().json(Res::new("Email already exists"))
            }
            _ => {
                eprintln!("Error registering user: {:?}", e);
                HttpResponse::InternalServerError().json(Res::new("Unknown Error"))
            }
        },
    }
}
