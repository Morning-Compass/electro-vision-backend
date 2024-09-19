#[cfg(test)]
mod tests {
    use core::panic;

    use crate::auth;

    const TEST_EMAIL: &str = "tomek@el-jot.eu";

    fn generate_token() -> String {
        match auth::jwt::generate(&TEST_EMAIL) {
            Ok(token) => {
                print!("token: {}", token)
            }
            Err(e) => panic!("Error generating token"),
        }
    }
}
