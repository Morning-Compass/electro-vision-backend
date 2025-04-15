-- -- Create roles table
-- CREATE TABLE IF NOT EXISTS roles (
--     id SERIAL PRIMARY KEY,
--     name VARCHAR NOT NULL DEFAULT 'USER'
-- );

-- -- Create confirmation_tokens table
-- CREATE TABLE IF NOT EXISTS confirmation_tokens (
--     id SERIAL PRIMARY KEY,
--     user_email VARCHAR NOT NULL,
--     token VARCHAR NOT NULL,
--     created_at TIMESTAMP NOT NULL,
--     expires_at TIMESTAMP NOT NULL,
--     confirmed_at TIMESTAMP NULL
-- );

-- CREATE TABLE IF NOT EXISTS password_reset_tokens (
--     id SERIAL PRIMARY KEY,
--     user_email VARCHAR NOT NULL,
--     token VARCHAR NOT NULL,
--     created_at TIMESTAMP NOT NULL,
--     expires_at TIMESTAMP NOT NULL,
--     confirmed_at TIMESTAMP NULL
-- );

-- -- Create users table
-- CREATE TABLE IF NOT EXISTS users (
--     id SERIAL PRIMARY KEY,
--     username VARCHAR NOT NULL,
--     email VARCHAR NOT NULL UNIQUE,
--     password VARCHAR NOT NULL,
--     created_at TIMESTAMP NOT NULL,
--     account_valid BOOLEAN NOT NULL
-- );

-- -- Create user_roles table
-- CREATE TABLE IF NOT EXISTS user_roles (
--     user_id INT NOT NULL,
--     role_id INT NOT NULL,
--     PRIMARY KEY (user_id, role_id),
--     FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
--     FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE
-- );


-- Create sequences with IF NOT EXISTS
CREATE SEQUENCE IF NOT EXISTS users_id_seq AS integer;
ALTER SEQUENCE IF EXISTS users_id_seq OWNER TO postgres;

CREATE SEQUENCE IF NOT EXISTS workspace_id_seq AS integer;
ALTER SEQUENCE IF EXISTS workspace_id_seq OWNER TO postgres;

CREATE SEQUENCE IF NOT EXISTS workspace_owner_id_seq AS integer;
ALTER SEQUENCE IF EXISTS workspace_owner_id_seq OWNER TO postgres;

CREATE SEQUENCE IF NOT EXISTS tasks_worker_id_seq AS integer;
ALTER SEQUENCE IF EXISTS tasks_worker_id_seq OWNER TO postgres;

CREATE SEQUENCE IF NOT EXISTS phone_dial_codes_country_seq AS integer;
ALTER SEQUENCE IF EXISTS phone_dial_codes_country_seq OWNER TO postgres;

-- Create diesel migrations table first
CREATE TABLE IF NOT EXISTS __diesel_schema_migrations
(
    version varchar(50)                         not null
        primary key,
    run_on  timestamp default CURRENT_TIMESTAMP not null
);

ALTER TABLE IF EXISTS __diesel_schema_migrations
    owner to postgres;

-- Create roles table
CREATE TABLE IF NOT EXISTS roles
(
    id   serial
        primary key,
    name varchar default 'USER'::character varying not null
);

ALTER TABLE IF EXISTS roles
    owner to postgres;

-- Create auth_users table
CREATE TABLE IF NOT EXISTS auth_users
(
    id            integer default nextval('users_id_seq'::regclass) not null
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

ALTER TABLE IF EXISTS auth_users
    owner to postgres;

ALTER SEQUENCE IF EXISTS users_id_seq OWNED BY auth_users.id;

-- Create password_reset_tokens table
CREATE TABLE IF NOT EXISTS password_reset_tokens
(
    id           serial
        primary key,
    user_email   varchar   not null
        constraint password_reset_tokens_auth_user_email_fk
            references auth_users (email),
    token        varchar   not null,
    created_at   timestamp not null,
    expires_at   timestamp not null,
    confirmed_at timestamp
);

ALTER TABLE IF EXISTS password_reset_tokens
    owner to postgres;

-- Create confirmation_tokens table
CREATE TABLE IF NOT EXISTS confirmation_tokens
(
    id           serial
        primary key,
    user_email   varchar   not null
        constraint confirmation_tokens_auth_user_email_fk
            references auth_users (email),
    token        varchar   not null,
    created_at   timestamp not null,
    expires_at   timestamp not null,
    confirmed_at timestamp
);

ALTER TABLE IF EXISTS confirmation_tokens
    owner to postgres;

-- Create user_roles table
CREATE TABLE IF NOT EXISTS user_roles
(
    user_id integer not null
        references auth_users
            on delete cascade,
    role_id integer not null
        references roles
            on delete cascade,
    primary key (user_id, role_id)
);

ALTER TABLE IF EXISTS user_roles
    owner to postgres;

-- Create importance table
CREATE TABLE IF NOT EXISTS importance
(
    id   integer not null
        constraint importance_pk
            primary key,
    name varchar(20)
);

ALTER TABLE IF EXISTS importance
    owner to postgres;

-- Create status table
CREATE TABLE IF NOT EXISTS status
(
    id   serial
        constraint status_pk
            primary key,
    name varchar(20) not null
);

ALTER TABLE IF EXISTS status
    owner to postgres;

-- Create workspace_roles table
CREATE TABLE IF NOT EXISTS workspace_roles
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

ALTER TABLE IF EXISTS workspace_roles
    owner to postgres;

-- Create ev_subscriptions table
CREATE TABLE IF NOT EXISTS ev_subscriptions
(
    id           serial
        constraint ev_subscriptions_pk
            primary key,
    subscription varchar(20) not null
);

ALTER TABLE IF EXISTS ev_subscriptions
    owner to postgres;

-- Create workspaces table
CREATE TABLE IF NOT EXISTS workspaces
(
    id                 integer   default nextval('workspace_id_seq'::regclass)       not null
        constraint workspace_pk
            primary key,
    plan_file_name     varchar(150)                                                  not null,
    start_date         timestamp default now()                                       not null,
    finish_date        timestamp,
    geolocation        varchar(40),
    owner_id           integer   default nextval('workspace_owner_id_seq'::regclass) not null
        constraint workspace_auth_user_id_fk
            references auth_users,
    ev_subscription_id serial
        constraint workspaces_ev_subscriptions_id_fk
            references ev_subscriptions
);

ALTER TABLE IF EXISTS workspaces
    owner to postgres;

ALTER SEQUENCE IF EXISTS workspace_id_seq OWNED BY workspaces.id;
ALTER SEQUENCE IF EXISTS workspace_owner_id_seq OWNED BY workspaces.owner_id;

-- Create tasks table
CREATE TABLE IF NOT EXISTS tasks
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
    worker_id              integer   default nextval('tasks_worker_id_seq'::regclass)
        constraint tasks_auth_user_id_fk_2
            references auth_users,
    description            text,
    description_multimedia bytea,
    assignment_date        timestamp default now() not null,
    due_date               timestamp,
    status_id              serial
        constraint tasks_status_id_fk
            references status
            on update cascade on delete cascade
);

ALTER TABLE IF EXISTS tasks
    owner to postgres;

ALTER SEQUENCE IF EXISTS tasks_worker_id_seq OWNED BY tasks.worker_id;

-- Create tasks_category table
CREATE TABLE IF NOT EXISTS tasks_category
(
    id           serial
        constraint tasks_category_pk
            primary key,
    workspace_id serial
        constraint tasks_category_workspaces_id_fk
            references workspaces,
    name         varchar(50) not null
);

ALTER TABLE IF EXISTS tasks_category
    owner to postgres;

-- Create problems table
CREATE TABLE IF NOT EXISTS problems
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
            references auth_users,
    problem_multimedia bytea
);

ALTER TABLE IF EXISTS problems
    owner to postgres;

-- Create worker_workspace_data table
CREATE TABLE IF NOT EXISTS worker_workspace_data
(
    employer_id   serial,
    user_id       serial
        constraint worker_workspace_data_pk
            primary key
        constraint worker_workspace_data_auth_users_id_fk
            references auth_users,
    working_since timestamp default now() not null
);

ALTER TABLE IF EXISTS worker_workspace_data
    owner to postgres;

-- Create positions table
CREATE TABLE IF NOT EXISTS positions
(
    id           serial
        constraint positions_pk
            primary key,
    workspace_id integer
        constraint positions_auth_users_id_fk
            references auth_users,
    name         varchar(50)
);

ALTER TABLE IF EXISTS positions
    owner to postgres;

-- Create phone_dial_codes table
CREATE TABLE IF NOT EXISTS phone_dial_codes
(
    id      serial
        constraint phone_dial_codes_pk
            primary key,
    code    serial,
    country varchar(50) default nextval('phone_dial_codes_country_seq'::regclass) not null
);

ALTER TABLE IF EXISTS phone_dial_codes
    owner to postgres;

ALTER SEQUENCE IF EXISTS phone_dial_codes_country_seq OWNED BY phone_dial_codes.country;

-- Create countries table
CREATE TABLE IF NOT EXISTS countries
(
    id   serial
        constraint countries_pk
            primary key,
    name varchar(50) not null
);

ALTER TABLE IF EXISTS countries
    owner to postgres;

-- Create full_users table
CREATE TABLE IF NOT EXISTS full_users
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
            references countries,
    title                varchar(50),
    education            varchar(100),
    birth_date           timestamp   not null,
    account_bank_number  varchar(70),
    photo                bytea
);

ALTER TABLE IF EXISTS full_users
    owner to postgres;

-- Create workspace_users table
CREATE TABLE IF NOT EXISTS workspace_users
(
    user_id             serial
        constraint workspace_users_pk
            primary key
        constraint workspace_users___fk
            references auth_users,
    workspace_id        serial
        constraint workspace_users_workspaces_id_fk
            references workspaces,
    plane_file_cut_name varchar(150) not null,
    workspace_role_id   serial
        constraint workspace_users_workspace_roles_id_fk
            references workspace_roles,
    position_id         serial
        constraint workspace_users_positions_id_fk
            references positions,
    checkin_time        time,
    checkout_time       time
);

ALTER TABLE IF EXISTS workspace_users
    owner to postgres;

-- Create conversations table
CREATE TABLE IF NOT EXISTS conversations
(
    id         serial
        constraint conversations_pk
            primary key,
    name       varchar(70) not null,
    created_at timestamp default now()
);

ALTER TABLE IF EXISTS conversations
    owner to postgres;

-- Create messages table
CREATE TABLE IF NOT EXISTS messages
(
    id              serial,
    conversation_id serial
        constraint messages_conversations_id_fk
            references conversations,
    sender_id       serial
        constraint messages_auth_users_id_fk
            references auth_users,
    body            text                    not null,
    read            boolean   default false not null,
    created_at      timestamp default now() not null
);

ALTER TABLE IF EXISTS messages
    owner to postgres;

-- Create conversation_participants table
CREATE TABLE IF NOT EXISTS conversation_participants
(
    conversation_id serial,
    user_id         serial,
    constraint conversation_participants_pk
        primary key (user_id, conversation_id)
);

ALTER TABLE IF EXISTS conversation_participants
    owner to postgres;

-- Create users_citizenships table
CREATE TABLE IF NOT EXISTS users_citizenships
(
    user_id    serial
        constraint users_citizenships_auth_users_id_fk
            references auth_users,
    country_id serial
        constraint users_citizenships_countries_id_fk
            references countries,
    constraint users_citizenships_pk
        primary key (country_id, user_id)
);

ALTER TABLE IF EXISTS users_citizenships
    owner to postgres;

-- Create functions
CREATE OR REPLACE FUNCTION diesel_manage_updated_at(_tbl regclass) RETURNS void
    LANGUAGE plpgsql
AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$;

ALTER FUNCTION diesel_manage_updated_at(regclass) OWNER TO postgres;

CREATE OR REPLACE FUNCTION diesel_set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$;

ALTER FUNCTION diesel_set_updated_at() OWNER TO postgres;

CREATE OR REPLACE FUNCTION init_status_values() RETURNS void
    LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO status (name) VALUES
    ('untouched'),
    ('in progress'),
    ('finished'),
    ('attendance needed')
    ON CONFLICT DO NOTHING;
END;
$$;

ALTER FUNCTION init_status_values() OWNER TO postgres;

-- Initialize status values
SELECT init_status_values();
