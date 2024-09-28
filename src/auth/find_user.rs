use diesel::QueryDsl;
use diesel::prelude::*;
use diesel::{result::Error as DieselError, ExpressionMethods};
use crate::models::User;
use crate::user::list_users;
use crate::{est_conn, schema, DPool};

pub struct FindData {
    pub is_found: Result<bool, DieselError>,
}

pub trait Find {
    async fn find_by_email(email: String, pool: DPool) -> Result<bool, DieselError>;
}

impl Find for FindData {
    async fn find_by_email(_email: String, pool: DPool) -> Result<bool, DieselError> {
        use schema::users::dsl::*;
        let conn = &mut est_conn(pool.clone());
        let user_data = list_users(1, pool).await;
        let is_found = diesel::select(diesel::dsl::exists(users.select(email).filter(email.eq(_email)))).get_result(conn);
        println!("\n\nUser data? {:?} \n\n", user_data);
        println!("\n\nWas found? {:?} \n\n", is_found);
        match is_found {
            Ok(true) => {
                println!("User found.");
                Ok(true)
            },
            Ok(false) => {
                println!("User not found.");
                Ok(false)
            },
            Err(e) => Err(e),
        }
    }
}
