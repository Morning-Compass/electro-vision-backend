use std::usize;

use chrono::Utc;
use diesel::result::Error as DieselError;
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods, RunQueryDsl, OptionalExtension, PgConnection};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use crate::models::User;
use crate::schema;
use crate::{constants::JWT_EXPIRATION_TIME, est_conn, schema::users::dsl::*, DPool};

fn verify_email(user_email: &str, pool: DPool) -> Result<bool, DieselError> {
    let exists = users
        .filter(email.eq(user_email.to_string()))
        .first::<User>(&mut est_conn(pool))
        .optional()?;

    match exists {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(e)
        }
    }
}

fn verify_date(iat: usize, exp: usize) -> bool {
    if exp - iat != JWT_EXPIRATION_TIME as usize {
        return false;
    }
    if Utc::now().timestamp() as usize > exp {
        return false;
    }
    true
}

pub fn verify(token: &str, pool: DPool) -> bool {
    match super::decode(token.to_string()) {
        Ok(token) => {
            if !verify_date(token.claims.iat, token.claims.exp) {
                return false;
            }

            match verify_email(&token.claims.email, pool) {
                Ok(true) => true,
                Ok(false) => {
                    eprintln!("Email not found");
                    false
                }
                Err(e) => {
                    eprintln!("Database error: {:?}", e);
                    false
                }
            }
        }
        Err(e) => {
            eprintln!("Error verifying token: {:?}", e);
            false
        }
    }
}
