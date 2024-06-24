use chrono::{DateTime, Utc};
use diesel::prelude::*;

#[allow(unused)]
#[derive(Queryable, Debug)]
#[diesel(table_name = create::schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub account_valid: bool,
}
