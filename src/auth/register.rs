use crate::models::User;
use crate::user::NoIdUser;
use crate::{est_conn, response, DPool, auth, schema, models};
use actix_web::web;
use actix_web::{post, web::Json, HttpResponse};
use diesel::dsl::{insert_into, select};
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error;
use serde_derive::Deserialize;
use auth::confirmation_token::token::ConfirmationToken;
use crate::auth::confirmation_token::token::Cft;
use crate::schema::roles::dsl::roles;
use crate::schema::roles::{id, name};
use crate::schema::user_roles::dsl::user_roles;
use crate::schema::user_roles::{role_id, user_id};

#[derive(Deserialize, Clone)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

type Register = response::Response<String>;

pub async fn insert_user(new_user: NoIdUser, pool: DPool) -> Result<User, Error> {
    use crate::schema::users::dsl::*;

    let hashed_passowrd = match bcrypt::hash(new_user.password, bcrypt::DEFAULT_COST) {
        Ok(hp) => hp,
        Err(_) => return Err(diesel::result::Error::RollbackTransaction),
    };

    match diesel::insert_into(users)
        .values((
            username.eq(new_user.username),
            email.eq(new_user.email),
            password.eq(hashed_passowrd),
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

pub async fn insert_user_roles(usr_id: i32, pool :DPool) -> Result<String, Error> {
    use crate::schema::user_roles::dsl::*;
    use crate::schema::roles::dsl::{roles, name as role_name};
    use diesel::result::Error;

    let mut conn = est_conn(pool);

    let role_id_value: i16 = roles
        .filter(role_name.eq("USER"))
        .select(crate::schema::roles::dsl::id)
        .first::<i16>(&mut conn)?;

    match diesel::insert_into(user_roles)
        .values((
            user_id.eq(usr_id),
            role_id.eq(role_id_value)
        ))
        .execute(&mut conn) {
        Ok(_) => Ok("User role assigned successfully".to_string()),
        Err(e) => {
            eprintln!("Error inserting user_roles: {:?}", e);
            Err(e)
        }
    }
}

#[post("/register")]
pub async fn register(request: Json<RegisterRequest>, pool: DPool) -> HttpResponse {
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
            match insert_user_roles(usr.id, pool.clone()).await {
                Ok(_) => {
                    match <Cft as ConfirmationToken>::new(request.email.clone(), pool) {
                        Ok(_) => HttpResponse::Ok().json(Register {
                            response: "User registered successfully!".to_string(),
                        }),
                        Err(e) => {
                            eprintln!("Error while creating token: {:?}", e);
                            HttpResponse::InternalServerError().json(Register {
                                response: "User registered successfully but confirmation token failed to be created".to_string(),
                            })
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error inserting user roles: {:?}", e);
                    HttpResponse::InternalServerError().json(Register {
                        response: "Error inserting user roles".to_string(),
                    })
                }
            }
        },
        Err(e) => match e {
            Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                if let Some(existing_email) = info.details() {
                    eprintln!("Email already exists: {}", existing_email);
                }
                HttpResponse::BadRequest().json(Register {
                    response: "Email already exists".to_string(),
                })
            }
            _ => {
                eprintln!("Error registering user: {:?}", e);
                HttpResponse::InternalServerError().json(Register {
                    response: "Error registering user".to_string(),
                })
            }
        },
    }
}
