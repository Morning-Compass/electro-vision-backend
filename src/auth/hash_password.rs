use bcrypt::hash;

use crate::constants::HASH_COST;

pub struct HashPassword {
    pub hashed_password: String,
}

pub trait Hash {
    async fn hash_password(password: String) -> String;
}

impl Hash for HashPassword {
    async fn hash_password(password: String) -> String {
        match hash(password, HASH_COST as u32) {
            Ok(hashed_password) => hashed_password,
            Err(e) => e.to_string(),
        }
    }
}
