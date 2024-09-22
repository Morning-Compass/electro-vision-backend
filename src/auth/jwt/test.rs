#[cfg(test)]
mod tests {
    use core::panic;

    use jsonwebtoken::TokenData;

    use crate::auth::{self, jwt::{jwt_decode, Claims}};

    const TEST_EMAIL: &str = "tomek@el-jot.eu";

    fn generate_token() -> String {
        match auth::jwt::generate(&TEST_EMAIL) {
            Ok(token) => {
                println!("token: {}", token);
                token
            }
            Err(e) => panic!("Error generating token, {}", e),
        }
    }

    fn decode() -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
        match jwt_decode(generate_token()) {
            Ok(claims) => {
                println!("Decoded claims: {:?}", claims.claims);
                Ok(claims)
            },
            Err(e) => {
                println!("Failed to decode token: {:?}", e);
                Err(e)
            }
        }
    }
}
