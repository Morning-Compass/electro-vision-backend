use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::UserWithRoles;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseUser {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub account_valid: bool,
    pub roles: Vec<String>,
    pub token: String,
}

impl ResponseUser {
    pub fn new(user: UserWithRoles) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            account_valid: user.account_valid,
            roles: user.roles,
            token: user.token,
        }
    }
}
