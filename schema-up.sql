create table roles
(
    id   serial
        primary key,
    name varchar default 'USER'::character varying not null
);

alter table roles
    owner to postgres;

create table auth_users
(
    id            serial
        constraint users_pkey
            primary key,
    username      varchar                                           not null,
    email         varchar                                           not null
        constraint users_email_key
            unique,
    password      varchar                                           not null,
    created_at    timestamp                                         not null,
    account_valid boolean                                           not null
);

alter table auth_users
    owner to postgres;

create table password_reset_tokens
(
    id           serial
        primary key,
    user_email   varchar   not null
        constraint password_reset_tokens_auth_user_email_fk
            references auth_users (email)
            on update cascade on delete cascade,
    token        varchar   not null,
    created_at   timestamp not null,
    expires_at   timestamp not null,
    confirmed_at timestamp
);

alter table password_reset_tokens
    owner to postgres;

create table confirmation_tokens
(
    id           serial
        primary key,
    user_email   varchar   not null
        constraint confirmation_tokens_auth_user_email_fk
            references auth_users (email)
            on update cascade on delete cascade,
    token        varchar   not null,
    created_at   timestamp not null,
    expires_at   timestamp not null,
    confirmed_at timestamp
);

alter table confirmation_tokens
    owner to postgres;

create table user_roles
(
    user_id serial not null
        references auth_users
            on update cascade on delete cascade,
    role_id serial not null
        references roles
            on update cascade on delete cascade,
    primary key (user_id, role_id)
);

alter table user_roles
    owner to postgres;

create table importance
(
    id  serial
        constraint importance_pk
            primary key,
    name varchar(20)
);

alter table importance
    owner to postgres;

create table status
(
    id  serial
        constraint status_pk
            primary key,
    name varchar(20) not null
);

alter table status
    owner to postgres;

create table workspace_roles
(
    id      serial
        constraint workspace_roles_pk
            primary key,
    user_id serial
        constraint workspace_roles_auth_users_id_fk
            references auth_users
            on update cascade on delete cascade,
    name    varchar(50) not null
);

alter table workspace_roles
    owner to postgres;

create table ev_subscriptions
(
    id           serial
        constraint ev_subscriptions_pk
            primary key,
    subscription varchar(20) not null
);

alter table ev_subscriptions
    owner to postgres;

create table workspaces
(
    id                 serial
        constraint workspace_pk
            primary key,
    plan_file_name     varchar(150)                                                  not null,
    start_date         timestamp default now()                                       not null,
    finish_date        timestamp,
    geolocation        varchar(40),
    owner_id           serial
        constraint workspace_auth_user_id_fk
            references auth_users
            on update cascade on delete cascade,
    ev_subscription_id serial
        constraint workspaces_ev_subscriptions_id_fk
            references ev_subscriptions
            on update cascade on delete cascade,
    name               varchar(60)                                                   not null
);

alter table workspaces
    owner to postgres;

create table tasks
(
    id                     serial
        constraint tasks_pk
            primary key,
    workspace_id           serial
        constraint tasks_workspaces_id_fk
            references workspaces
            on update cascade on delete cascade,
    assigner_id            serial
        constraint tasks_auth_user_id_fk
            references auth_users
            on update cascade on delete cascade,
    worker_id              serial
        constraint tasks_auth_user_id_fk_2
            references auth_users
            on update cascade on delete cascade,
    description            text,
    description_multimedia bytea,
    assignment_date        timestamp default now() not null,
    due_date               timestamp,
    status_id              serial
        constraint tasks_status_id_fk
            references status
            on update cascade on delete cascade
);

alter table tasks
    owner to postgres;

create table tasks_category
(
    id           serial
        constraint tasks_category_pk
            primary key,
    workspace_id serial
        constraint tasks_category_workspaces_id_fk
            references workspaces
            on update cascade on delete cascade,
    name         varchar(50) not null
);

alter table tasks_category
    owner to postgres;

create table problems
(
    id                 serial
        constraint problems_pk
            primary key,
    worker_id          serial
        constraint problems_auth_users_id_fk
            references auth_users
            on update cascade on delete cascade,
    description        text,
    mentor_id          serial
        constraint problems_auth_users_id_fk_2
            references auth_users
            on update cascade on delete cascade,
    problem_multimedia bytea
);

alter table problems
    owner to postgres;

create table worker_workspace_data
(
    employer_id   serial,
    user_id       serial
        constraint worker_workspace_data_pk
            primary key
        constraint worker_workspace_data_auth_users_id_fk
            references auth_users
            on update cascade on delete cascade,
    working_since timestamp default now() not null
);

alter table worker_workspace_data
    owner to postgres;

create table positions
(
    id           serial
        constraint positions_pk
            primary key,
    workspace_id serial
        constraint positions_auth_users_id_fk
            references auth_users
            on update cascade on delete cascade,
    name         varchar(50)
);

alter table positions
    owner to postgres;

create table phone_dial_codes
(
    id      serial
        constraint phone_dial_codes_pk
            primary key,
    code    varchar(6) not null,
    country varchar(50)  not null
);

alter table phone_dial_codes
    owner to postgres;

create table countries
(
    id   serial
        constraint countries_pk
            primary key,
    name varchar(50) not null
);

alter table countries
    owner to postgres;

create table full_users
(
    user_id              serial
        constraint full_users_pk
            primary key,
    phone                varchar(10) not null,
    phonde_dial_code_id  serial
        constraint full_users_phone_dial_codes_id_fk
            references phone_dial_codes
            on update cascade on delete cascade,
    countru_of_origin_id serial
        constraint full_users_countries_id_fk
            references countries
            on update cascade on delete cascade,
    title                varchar(50),
    education            varchar(100),
    birth_date           timestamp   not null,
    account_bank_number  varchar(70),
    photo                bytea
);

alter table full_users
    owner to postgres;

create table workspace_users
(
    user_id             serial
        constraint workspace_users_pk
            primary key
        constraint workspace_users___fk
            references auth_users
            on update cascade on delete cascade,
    workspace_id        serial
        constraint workspace_users_workspaces_id_fk
            references workspaces
            on update cascade on delete cascade,
    plane_file_cut_name varchar(150),
    workspace_role_id   serial
        constraint workspace_users_workspace_roles_id_fk
            references workspace_roles
            on update cascade on delete cascade,
    position_id         integer NULL
        constraint workspace_users_positions_id_fk
            references positions
            on update cascade on delete cascade,
    checkin_time        time,
    checkout_time       time
);

alter table workspace_users
    owner to postgres;

create table conversations
(
    id         serial
        constraint conversations_pk
            primary key,
    name       varchar(70) not null,
    created_at timestamp default now() not null
);

alter table conversations
    owner to postgres;

create table messages
(
    id              serial
        constraint messages_pk
            primary key,
    conversation_id serial
        constraint messages_conversations_id_fk
            references conversations
            on update cascade on delete cascade,
    sender_id       serial
        constraint messages_auth_users_id_fk
            references auth_users
            on update cascade on delete cascade,
    body            text                    not null,
    read            boolean   default false not null,
    created_at      timestamp default now() not null
);

alter table messages
    owner to postgres;

create table conversation_participants
(
    conversation_id serial,
    user_id         serial,
    constraint conversation_participants_pk
        primary key (user_id, conversation_id)
);

alter table conversation_participants
    owner to postgres;

create table users_citizenships
(
    user_id    serial
        constraint users_citizenships_auth_users_id_fk
            references auth_users
            on update cascade on delete cascade,
    country_id serial
        constraint users_citizenships_countries_id_fk
            references countries
            on update cascade on delete cascade,
    constraint users_citizenships_pk
        primary key (country_id, user_id)
);

alter table users_citizenships
    owner to postgres;

create table attendance
(
    id             serial
        constraint attendance_pk
            primary key,
    user_id        serial
        constraint attendance_auth_users_id_fk
            references auth_users
            on update cascade on delete cascade,
    date           date not null,
    checkin        time not null,
    checkin_photo  bytea,
    checkout       time,
    checkout_photo bytea,
    workspace_id   serial
        constraint attendance_workspaces_id_fk
            references workspaces
            on update cascade on delete cascade
);


create table workspace_invitations
(
    id           serial
        constraint workspace_invitations_pk
            primary key,
    user_email   varchar                 not null
        constraint workspace_invitations_auth_users_email_fk
            references auth_users (email)
            on update cascade on delete cascade,
    token        varchar                 not null,
    created_at   timestamp default now() not null,
    expires_at   timestamp               not null,
    confirmed_at timestamp,
    workspace_id serial not null
        constraint workspace_invitations_workspaces_id_fk
            references workspaces
            on update cascade on delete cascade
);


alter table attendance
    owner to postgres;
