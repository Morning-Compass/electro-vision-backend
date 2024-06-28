use chrono::{NaiveDateTime, TimeDelta, Utc};
use diesel::{ExpressionMethods, Insertable, RunQueryDsl};
use diesel::result::Error;

use crate::{
    auth::confirmation_token::token,
    constants::CONFIRMATION_TOKEN_EXIPIRATION_TIME,
    est_conn, response,
    schema::{confirmation_tokens, users::created_at},
    DPool,
};

struct Cft {
    pub user_email: String,
    pub token: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

trait ConfirmationToken {
    fn new(email: String, pool: DPool) -> Result<String, diesel::result::Error>;
    fn confirm(token: Cft, pool: DPool) -> Result<String, diesel::result::Error>;
    fn send(
        token: String,
        username: String,
        email: String,
        pool: DPool,
    ) -> Result<String, diesel::result::Error>;
}

impl ConfirmationToken for Cft {
    fn new(email: String, pool: DPool) -> Result<String, diesel::result::Error> {
        use crate::schema::confirmation_tokens::dsl::*;

        let ctoken = Cft {
            user_email: email,
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

    fn confirm(token: Cft, pool: DPool) -> Result<String, Error> {
        todo!()
    }

    fn send(token: String, username: String, email: String, pool: DPool) -> Result<String, Error> {
        todo!()
    }
}
