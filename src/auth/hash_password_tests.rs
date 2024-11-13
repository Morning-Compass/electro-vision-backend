#[cfg(test)]
mod tests {
    use crate::{
        auth::hash_password::{Hash, HashPassword},
        constants::TEST_PASSWORD,
    };

    #[actix_web::test]
    async fn test_hash_password() {
        let password = TEST_PASSWORD.to_string();
        let hashed_password = HashPassword::hash_password(password.clone()).await;

        println!(
            "\n PASSWORD: {:?} \n HASHED PASSWORD: {:?}",
            password.clone(),
            hashed_password
        );
    }
}
