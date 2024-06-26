use crate::schema::*;
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(unused)]
#[derive(Queryable, Debug, Serialize, Deserialize)]
#[diesel(table_name = create::schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at: chrono::DateTime<Utc>,
    pub account_valid: bool,
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password: &'a str,
    pub created_at: chrono::DateTime<Utc>,
    pub account_valid: bool,
}
