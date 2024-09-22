#[cfg(test)]
mod tests {
    use core::panic;

    use crate::auth::{self, jwt::jwt_decode};

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

    #[test]
    fn decode() {
        match jwt_decode(generate_token()) {
            Ok(claims) => {
                println!("Decoded claims: {:?}", claims.claims);
            }
            Err(e) => {
                panic!("Failed to decode token: {:?}", e);
            }
        }
    }
}
