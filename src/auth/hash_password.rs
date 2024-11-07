use bcrypt::hash;

pub struct HashPassword {
    pub hashed_password: String,
}

pub trait Hash {
    async fn hash_password(password: String) -> String;
}

impl Hash for HashPassword {
    async fn hash_password(password: String) -> String {
        match hash(password, 10) {
            Ok(hashed_password) => hashed_password,
            Err(e) => e.to_string(),
        }
    }
}
