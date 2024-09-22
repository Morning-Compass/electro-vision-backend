#[cfg(test)]
mod tests {
    use chrono::Utc;

    pub use crate::auth::login::login::UserWithRoles;
    pub use crate::models::{User, Role};

    fn login_with_roles() {
        const USER_DATA: UserWithRoles = UserWithRoles::new(User{id: 1, username: String::from("Oaza_spokoju"), email: String::from("oaza_spokoju@gmail.com"), password: String::from("oazaSpokoju123!"), created_at: Utc::now().naive_utc(), account_valid: false}, Vec::from([Role{id: 1, name: String::from("SpokojNasUratuje")}]));
    }
}
