use crate::auth::jwt::Claims;
use crate::constants::JWT_EXPIRATION_TIME;
use chrono::Utc;
use dotenv;
use jsonwebtoken::{encode, EncodingKey, Header};

pub fn generate(email: &str) -> Result<String, Box<dyn std::error::Error>> {
    let now = Utc::now();
    let expiration = now + chrono::Duration::seconds(JWT_EXPIRATION_TIME);
    let secret_key = dotenv::var("JWT_SECRET").expect("JWT secret must be set");

    let claims = Claims {
        iat: now.timestamp() as usize,
        exp: expiration.timestamp() as usize,
        email: email.to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )?;

    Ok(token)
}
