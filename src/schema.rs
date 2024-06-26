// @generated automatically by Diesel CLI.

diesel::table! {
    confirmation_tokens (id) {
        id -> Serial,
        user_email -> Varchar,
        token -> Varchar,
        created_at -> Timestamp,
        expires_at -> Timestamp,
        confirmed_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    roles (id) {
        id -> Serial,
        name -> Varchar,
    }
}

diesel::table! {
    user_roles (user_id, role_id) {
        user_id -> Serial,
        role_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Serial,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
        account_valid -> Bool,
    }
}

diesel::joinable!(user_roles -> roles (role_id));
diesel::joinable!(user_roles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(confirmation_tokens, roles, user_roles, users);
