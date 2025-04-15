use crate::models::User;
use crate::{est_conn, schema, DPool};
use diesel::prelude::*;
use diesel::QueryDsl;
use diesel::{result::Error as DieselError, ExpressionMethods};

pub struct FindData {}

pub trait Find {
    async fn exists_by_email(email: String, pool: DPool) -> Result<bool, DieselError>;
    async fn find_by_email(email: String, pool: DPool) -> Result<User, DieselError>;
}

impl Find for FindData {
    async fn exists_by_email(_email: String, pool: DPool) -> Result<bool, DieselError> {
        use schema::users::dsl::*;
        let conn = &mut est_conn(pool.clone());
        let is_found = diesel::select(diesel::dsl::exists(
            users.select(email).filter(email.eq(_email.clone())),
        ))
        .get_result::<bool>(conn);
        match is_found {
            Ok(true) => {
                println!("User found.");
                Ok(true)
            }
            Ok(false) => {
                println!("User not found.");
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }
    async fn find_by_email(_email: String, pool: DPool) -> Result<User, DieselError> {
        use schema::users::dsl::*;
        let conn = &mut est_conn(pool.clone());
        let user_data = users
            .filter(email.eq(_email))
            .select(User::as_select())
            .first(conn);
        match user_data {
            Ok(user) => {
                // println!("User data was found find_user.rs {:?}", user);
                Ok(user)
            }
            Err(e) => Err(e),
        }
    }
}
