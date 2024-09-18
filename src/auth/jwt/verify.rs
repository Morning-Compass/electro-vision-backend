use crate::{constants::JWT_EXPIRATION_TIME, est_conn, DPool};
use chrono::Utc;
use diesel::RunQueryDsl;
use diesel::{
    query_dsl::methods::FilterDsl,
    result::Error as DieselError,
    ExpressionMethods,
};
use std::usize;

fn verify_email(users_email: String, pool: DPool) -> Result<bool, DieselError> {
    use crate::schema::users::dsl::*;

    let exists = diesel::select(diesel::dsl::exists(users.filter(email.eq(users_email))))
        .get_result(&mut est_conn(pool.clone()));

    match exists {
        Ok(true) => Ok(true),
        Ok(false) => Ok(false),
        Err(e) => Err(e),
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
    match super::jwt_decode(token.to_string()) {
        Ok(token) => {
            if !verify_date(token.claims.iat, token.claims.exp) {
                return false;
            }

            match verify_email(token.claims.email, pool) {
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
