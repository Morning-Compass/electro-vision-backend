// @generated automatically by Diesel CLI.
diesel::table! {
    user (email) {
        id -> Integer,
        username -> VarChar,
        email -> VarChar,
        password -> VarChar,
        created_at -> Timestamp,
        account_valid -> Bool,
    }
}

diesel::table! {
    confirmation_tokens (id) {
        id -> Integer,
        user_email -> VarChar,
        token -> VarChar,
        created_at -> Timestamp,
        expires_at -> Timestamp,
        confirmed_at -> Timestamp,
    }
}

diesel::table! {
    roles (id) {
        id -> Int4,
        name -> VarChar,
    }
}

diesel::table! {
    user_roles (user_id) {
        user_id -> Integer,
        role_id -> Int4
    }
}

diesel::joinable!(confirmation_tokens -> user (user_email));

diesel::allow_tables_to_appear_in_same_query!(user, confirmation_tokens);
