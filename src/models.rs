use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub account_valid: bool,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    pub id: i16,
    pub name: String,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::user_roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserRole {
    pub user_id: i32,
    pub role_id: i16,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::confirmation_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ConfirmationToken {
    pub id: i32,
    pub user_email: String,
    pub token: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub confirmed_at: Option<NaiveDateTime>,
}

