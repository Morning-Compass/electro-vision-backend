use std::usize;

use crate::{constants::JWT_EXPIRATION_TIME, est_conn, DPool};
use diesel::{
    query_dsl::methods::{FilterDsl, SelectDsl},
    result::Error as DieselError,
    ExpressionMethods, QueryDsl, RunQueryDsl,
};

fn verify_email(user_email: &str, pool: DPool) -> Result<bool, DieselError> {
    use crate::schema::users::dsl::*;

    match exists {
        Some(_) => Ok(true),
        None => Ok(false),
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
