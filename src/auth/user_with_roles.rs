use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithRoles {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub account_valid: bool,
    pub roles: Vec<String>,
    pub token: String,
}

impl UserWithRoles {
    pub fn new(user: User, roles: Vec<String>, token: String) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            password: user.password,
            created_at: user.created_at,
            account_valid: user.account_valid,
            roles,
            token,
        }
    }
}
