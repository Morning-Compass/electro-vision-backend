#[cfg(test)]
mod tests {
    use chrono::Utc;

    pub use crate::auth::login::login::UserWithRoles;
    pub use crate::models::User;

    fn login_with_roles() {
        let user_data: UserWithRoles = UserWithRoles::new(
            User {
                id: 1,
                username: String::from("Oaza_spokoju"),
                email: String::from("oaza_spokoju@gmail.com"),
                password: String::from("oazaSpokoju123!"),
                created_at: Utc::now().naive_utc(),
                account_valid: false,
            },
            vec![String::from("Spokojna rola")],
        );
    }
}
