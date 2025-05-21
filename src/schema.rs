diesel::table! {
    attendance (id) {
        id -> Int4,
        user_id -> Int4,
        date -> Date,
        checkin -> Time,
        checkin_photo -> Nullable<Bytea>,
        checkout -> Nullable<Time>,
        checkout_photo -> Nullable<Bytea>,
        workspace_id -> Int4,
    }
}

diesel::table! {
    auth_users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
        account_valid -> Bool,
    }
}

diesel::table! {
    confirmation_tokens (id) {
        id -> Int4,
        user_email -> Varchar,
        token -> Varchar,
        created_at -> Timestamp,
        expires_at -> Timestamp,
        confirmed_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    conversation_participants (user_id, conversation_id) {
        conversation_id -> Int4,
        user_id -> Int4,
    }
}

diesel::table! {
    conversations (id) {
        id -> Int4,
        #[max_length = 70]
        name -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::table! {
    countries (id) {
        id -> Int4,
        #[max_length = 50]
        name -> Varchar,
        iso3 -> Nullable<Varchar>,
        numeric_code -> Nullable<Varchar>,
    }
}

diesel::table! {
    ev_subscriptions (id) {
        id -> Int4,
        #[max_length = 20]
        subscription -> Varchar,
    }
}

diesel::table! {
    full_users (user_id) {
        user_id -> Int4,
        #[max_length = 10]
        phone -> Varchar,
        phonde_dial_code_id -> Int4,
        country_of_origin_id -> Int4,
        #[max_length = 50]
        title -> Nullable<Varchar>,
        #[max_length = 100]
        education -> Nullable<Varchar>,
        birth_date -> Date,
        #[max_length = 70]
        account_bank_number -> Nullable<Varchar>,
        photo -> Nullable<Bytea>,
    }
}

diesel::table! {
    importance (id) {
        id -> Int4,
        #[max_length = 20]
        name -> Nullable<Varchar>,
    }
}

diesel::table! {
    messages (id) {
        id -> Int4,
        conversation_id -> Int4,
        sender_id -> Int4,
        body -> Text,
        read -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    password_reset_tokens (id) {
        id -> Int4,
        user_email -> Varchar,
        token -> Varchar,
        created_at -> Timestamp,
        expires_at -> Timestamp,
        confirmed_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    phone_dial_codes (id) {
        id -> Int4,
        #[max_length = 50]
        country_id -> Int4,
        #[max_length = 6]
        code -> Varchar,
    }
}

diesel::table! {
    positions (id) {
        id -> Int4,
        workspace_id -> Int4,
        #[max_length = 50]
        name -> Nullable<Varchar>,
    }
}

diesel::table! {
    problems (id) {
        id -> Int4,
        workspace_id -> Int4,
        worker_id -> Int4,
        description -> Nullable<Text>,
        mentor_id -> Int4,
        problem_multimedia_path -> Nullable<Varchar>,
    }
}

diesel::table! {
    roles (id) {
        id -> Int4,
        name -> Varchar,
    }
}

diesel::table! {
    status (id) {
        id -> Int4,
        #[max_length = 20]
        name -> Varchar,
    }
}

diesel::table! {
    tasks (id) {
        id -> Int4,
        workspace_id -> Int4,
        assigner_id -> Int4,
        worker_id -> Int4,
        description -> Nullable<Text>,
        description_multimedia_path -> Nullable<Varchar>,
        assignment_date -> Timestamp,
        due_date -> Nullable<Timestamp>,
        status_id -> Int4,
        title -> Varchar,
        category_id -> Int4,
        importance_id -> Int4,
    }
}

diesel::table! {
    tasks_category (id) {
        id -> Int4,
        workspace_id -> Int4,
        #[max_length = 50]
        name -> Varchar,
    }
}

diesel::table! {
    user_roles (user_id, role_id) {
        user_id -> Int4,
        role_id -> Int4,
    }
}

diesel::table! {
    users_citizenships (country_id, user_id) {
        user_id -> Int4,
        country_id -> Int4,
    }
}

diesel::table! {
    worker_workspace_data (user_id) {
        employer_id -> Int4,
        user_id -> Int4,
        working_since -> Timestamp,
    }
}

diesel::table! {
    workspace_invitations (id) {
        id -> Int4,
        user_email -> Varchar,
        token -> Varchar,
        created_at -> Timestamp,
        expires_at -> Timestamp,
        confirmed_at -> Nullable<Timestamp>,
        workspace_id -> Int4,
    }
}

diesel::table! {
    workspace_roles (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 50]
        name -> Varchar,
    }
}

diesel::table! {
    workspace_users (user_id) {
        user_id -> Int4,
        workspace_id -> Int4,
        #[max_length = 150]
        plane_file_cut_name -> Nullable<Varchar>,
        workspace_role_id -> Int4,
        position_id -> Nullable<Int4>,
        checkin_time -> Nullable<Time>,
        checkout_time -> Nullable<Time>,
    }
}

diesel::table! {
    workspaces (id) {
        id -> Int4,
        #[max_length = 150]
        plan_file_name -> Varchar,
        start_date -> Timestamp,
        finish_date -> Nullable<Timestamp>,
        #[max_length = 40]
        geolocation -> Nullable<Varchar>,
        owner_id -> Int4,
        ev_subscription_id -> Int4,
        #[max_length = 60]
        name -> Varchar,
    }
}

diesel::joinable!(attendance -> auth_users (user_id));
diesel::joinable!(attendance -> workspaces (workspace_id));
diesel::joinable!(full_users -> countries (country_of_origin_id));
diesel::joinable!(full_users -> phone_dial_codes (phonde_dial_code_id));
diesel::joinable!(messages -> auth_users (sender_id));
diesel::joinable!(messages -> conversations (conversation_id));
diesel::joinable!(positions -> auth_users (workspace_id));
diesel::joinable!(tasks -> status (status_id));
diesel::joinable!(tasks -> workspaces (workspace_id));
diesel::joinable!(tasks_category -> workspaces (workspace_id));
diesel::joinable!(user_roles -> auth_users (user_id));
diesel::joinable!(user_roles -> roles (role_id));
diesel::joinable!(users_citizenships -> auth_users (user_id));
diesel::joinable!(users_citizenships -> countries (country_id));
diesel::joinable!(worker_workspace_data -> auth_users (user_id));
diesel::joinable!(workspace_invitations -> workspaces (workspace_id));
diesel::joinable!(workspace_roles -> auth_users (user_id));
diesel::joinable!(workspace_users -> auth_users (user_id));
diesel::joinable!(workspace_users -> positions (position_id));
diesel::joinable!(workspace_users -> workspace_roles (workspace_role_id));
diesel::joinable!(workspace_users -> workspaces (workspace_id));
diesel::joinable!(workspaces -> auth_users (owner_id));
diesel::joinable!(workspaces -> ev_subscriptions (ev_subscription_id));

diesel::allow_tables_to_appear_in_same_query!(
    attendance,
    auth_users,
    confirmation_tokens,
    conversation_participants,
    conversations,
    countries,
    ev_subscriptions,
    full_users,
    importance,
    messages,
    password_reset_tokens,
    phone_dial_codes,
    positions,
    problems,
    roles,
    status,
    tasks,
    tasks_category,
    user_roles,
    users_citizenships,
    worker_workspace_data,
    workspace_invitations,
    workspace_roles,
    workspace_users,
    workspaces,
);
