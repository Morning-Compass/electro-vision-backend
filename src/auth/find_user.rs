use diesel::QueryDsl;
use diesel::prelude::*;
use diesel::{result::Error as DieselError, ExpressionMethods};

use crate::{est_conn, schema, DPool};

trait Find {
    fn find_by_email(_email: String, pool: DPool) -> Result<bool, DieselError>;
}

struct FindData {
    pub is_found: bool,
}

impl Find for FindData {
    fn find_by_email(_email: String, pool: DPool) -> Result<bool, DieselError> {
        use schema::users::dsl::*;
        let conn = &mut est_conn(pool.clone());

        let exists = diesel::select(diesel::dsl::exists(users.filter(email.eq(_email)))).get_result(conn);
        match exists {
            Ok(true) => Ok(true),
            Ok(false) => Ok(false),
            Err(e) => Err(e),
        }
    }
}
