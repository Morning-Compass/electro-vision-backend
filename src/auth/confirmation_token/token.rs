use chrono::{Duration, NaiveDateTime, TimeDelta, Utc};
use diesel::prelude::OptionalExtension;
use diesel::query_dsl::methods::FilterDsl;
use diesel::result::DatabaseErrorKind;
use diesel::{ExpressionMethods, RunQueryDsl}; // Import this for `optional` method

use crate::schema::users::account_valid;
use crate::{constants::CONFIRMATION_TOKEN_EXIPIRATION_TIME, est_conn, DPool};
use crate::{models, schema};

pub struct Cft {
    pub user_email: String,
    pub token: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

pub trait ConfirmationToken {
    fn new(email: String, pool: DPool) -> Result<String, diesel::result::Error>;
    fn confirm(token: String, pool: DPool) -> Result<String, diesel::result::Error>;
    fn send(
        token: String,
        username: String,
        email: String,
        pool: DPool,
    ) -> Result<String, diesel::result::Error>;
}

impl ConfirmationToken for Cft {
    fn new(u_email: String, pool: DPool) -> Result<String, diesel::result::Error> {
        use crate::schema::confirmation_tokens::dsl::*;

        let ctoken = Cft {
            user_email: u_email,
            token: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: Utc::now()
                .naive_utc()
                .checked_add_signed(TimeDelta::seconds(CONFIRMATION_TOKEN_EXIPIRATION_TIME))
                .ok_or_else(|| {
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UnableToSendCommand,
                        Box::new("Failed to set expiration time".to_string()),
                    )
                })?,
        };

        match diesel::insert_into(confirmation_tokens)
            .values((
                user_email.eq(ctoken.user_email),
                token.eq(ctoken.token),
                created_at.eq(ctoken.created_at),
                expires_at.eq(ctoken.expires_at),
            ))
            .execute(&mut est_conn(pool))
        {
            Ok(_) => Ok("Token inserted successfully".to_string()),
            Err(_) => Ok("Error inserting token".to_string()),
        }
    }

    fn confirm(_token: String, pool: DPool) -> Result<String, diesel::result::Error> {
        use crate::schema::confirmation_tokens::dsl::*;
        let mut conn = est_conn(pool);

        let db_token = match confirmation_tokens
            .filter(token.eq(&_token))
            .first::<models::ConfirmationToken>(&mut conn)
            .optional()?
        {
            Some(tok) => {
                if tok.confirmed_at.is_some() {
                    return Err(diesel::result::Error::AlreadyInTransaction); // defintelly after auth need to add custom error
                }
                let current_time = Utc::now().naive_utc();
                if current_time - Duration::seconds(CONFIRMATION_TOKEN_EXIPIRATION_TIME)
                    > tok.created_at
                {
                    Err(diesel::result::Error::NotFound) // Custom error message can be mapped later
                } else {
                    match diesel::update(
                        schema::confirmation_tokens::dsl::confirmation_tokens
                            .filter(schema::confirmation_tokens::dsl::token.eq(tok.token.clone())),
                    )
                    .set(schema::confirmation_tokens::confirmed_at.eq(Utc::now().naive_utc()))
                    .execute(&mut conn)
                    {
                        Ok(_) => Ok(tok),
                        Err(e) => {
                            eprintln!("Error updating token verified status: {:?}", e);
                            Err(diesel::result::Error::DatabaseError(
                                DatabaseErrorKind::UnableToSendCommand,
                                Box::new(format!("Error updating token: {}", e)),
                            ))
                        }
                    }
                }
            }
            None => Err(diesel::result::Error::NotFound),
        };

        match db_token {
            Ok(_) => {
                match diesel::update(
                    schema::users::dsl::users
                        .filter(schema::users::dsl::email.eq(db_token.unwrap().user_email)),
                )
                .set(account_valid.eq(true))
                .execute(&mut conn)
                {
                    Ok(_) => Ok("Account verified".to_string()),
                    Err(e) => Err(diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UnableToSendCommand,
                        Box::new(format!("Error while updating account: {}", e)),
                    )),
                }
            }
            Err(e) => Err(e),
        }
    }

    fn send(
        token: String,
        username: String,
        u_email: String,
        pool: DPool,
    ) -> Result<String, diesel::result::Error> {
        todo!()
    }
}
