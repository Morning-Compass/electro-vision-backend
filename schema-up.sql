-- up.sql
CREATE TABLE roles (
    id   serial PRIMARY KEY,
    name varchar DEFAULT 'USER'::character varying NOT NULL
);

CREATE TABLE auth_users (
    id            serial PRIMARY KEY,
    username      varchar NOT NULL,
    email         varchar NOT NULL UNIQUE,
    password      varchar NOT NULL,
    created_at    timestamp NOT NULL,
    account_valid boolean NOT NULL
);

CREATE TABLE confirmation_tokens (
    id           serial PRIMARY KEY,
    user_email   varchar NOT NULL REFERENCES auth_users(email) ON UPDATE CASCADE ON DELETE CASCADE,
    token        varchar NOT NULL,
    created_at   timestamp DEFAULT now() NOT NULL,
    expires_at   timestamp NOT NULL,
    confirmed_at timestamp
);

CREATE TABLE password_reset_tokens (
    id           serial PRIMARY KEY,
    user_email   varchar NOT NULL REFERENCES auth_users(email) ON UPDATE CASCADE ON DELETE CASCADE,
    token        varchar NOT NULL,
    created_at   timestamp DEFAULT now() NOT NULL,
    expires_at   timestamp NOT NULL,
    confirmed_at timestamp
);

CREATE TABLE importance (
    id   serial PRIMARY KEY,
    name varchar(20)
);

CREATE TABLE status (
    id   serial PRIMARY KEY,
    name varchar(20) NOT NULL
);

CREATE TABLE ev_subscriptions (
    id           serial PRIMARY KEY,
    subscription varchar(20) NOT NULL
);

CREATE TABLE countries (
    id   serial PRIMARY KEY,
    name varchar(50) NOT NULL,
    iso3 varchar(3) NOT NULL,
    numeric_code integer NOT NULL
);

CREATE TABLE phone_dial_codes (
    id SERIAL PRIMARY KEY,
    code    varchar(6) NOT NULL,
    country_id serial REFERENCES countries ON UPDATE CASCADE ON DELETE CASCADE,
    UNIQUE (code, country_id)
);

CREATE TABLE workspace_roles (
    id      serial PRIMARY KEY,
    user_id serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    name    varchar(50) NOT NULL
);

CREATE TABLE user_roles (
    user_id serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    role_id serial REFERENCES roles ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE workspaces (
    id                 serial PRIMARY KEY,
    plan_file_name     varchar(150) NOT NULL,
    start_date         timestamp DEFAULT now() NOT NULL,
    finish_date        timestamp,
    geolocation        varchar(40),
    owner_id           serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    ev_subscription_id serial REFERENCES ev_subscriptions ON UPDATE CASCADE ON DELETE CASCADE,
    name               varchar(60) NOT NULL,
    UNIQUE (owner_id, name)
);

CREATE TABLE tasks_category (
    id           serial PRIMARY KEY,
    workspace_id serial REFERENCES workspaces ON UPDATE CASCADE ON DELETE CASCADE,
    name         varchar(50) NOT NULL
);

CREATE TABLE tasks (
    id                     serial PRIMARY KEY,
    workspace_id           serial REFERENCES workspaces ON UPDATE CASCADE ON DELETE CASCADE,
    assigner_id            serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    worker_id              serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    description            text,
    description_multimedia bytea,
    assignment_date        timestamp DEFAULT now() NOT NULL,
    due_date               timestamp,
    status_id              serial REFERENCES status ON UPDATE CASCADE ON DELETE CASCADE,
    title                  varchar(50) NOT NULL,
    category_id            serial REFERENCES tasks_category ON UPDATE CASCADE ON DELETE CASCADE,
    importance_id          serial REFERENCES importance ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE problems (
    id                 serial PRIMARY KEY,
    worker_id          serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    description        text,
    mentor_id          serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    problem_multimedia bytea
);

CREATE TABLE worker_workspace_data (
    employer_id   serial,
    user_id       serial PRIMARY KEY REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    working_since timestamp DEFAULT now() NOT NULL
);

CREATE TABLE positions (
    id           serial PRIMARY KEY,
    workspace_id serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    name         varchar(50)
);

CREATE TABLE full_users (
    user_id              serial PRIMARY KEY,
    phone                varchar(10) NOT NULL,
    phonde_dial_code_id  serial REFERENCES phone_dial_codes ON UPDATE CASCADE ON DELETE CASCADE,
    country_of_origin_id serial REFERENCES countries ON UPDATE CASCADE ON DELETE CASCADE,
    title                varchar(50),
    education            varchar(100),
    birth_date           date NOT NULL,
    account_bank_number  varchar(70),
    photo                bytea
);

CREATE TABLE workspace_users (
    user_id             serial PRIMARY KEY REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    workspace_id        serial REFERENCES workspaces ON UPDATE CASCADE ON DELETE CASCADE,
    plane_file_cut_name varchar(150),
    workspace_role_id   serial REFERENCES workspace_roles ON UPDATE CASCADE ON DELETE CASCADE,
    position_id         integer NULL REFERENCES positions ON UPDATE CASCADE ON DELETE CASCADE,
    checkin_time        time,
    checkout_time       time
);

CREATE TABLE conversations (
    id         serial PRIMARY KEY,
    name       varchar(70) NOT NULL,
    created_at timestamp DEFAULT now() NOT NULL
);

CREATE TABLE messages (
    id              serial PRIMARY KEY,
    conversation_id serial REFERENCES conversations ON UPDATE CASCADE ON DELETE CASCADE,
    sender_id       serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    body            text NOT NULL,
    read            boolean DEFAULT false NOT NULL,
    created_at      timestamp DEFAULT now() NOT NULL
);

CREATE TABLE conversation_participants (
    conversation_id serial REFERENCES conversations ON UPDATE CASCADE ON DELETE CASCADE,
    user_id         serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (user_id, conversation_id)
);

CREATE TABLE users_citizenships (
    user_id    serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    country_id serial REFERENCES countries ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (country_id, user_id)
);

CREATE TABLE attendance (
    id             serial PRIMARY KEY,
    user_id        serial REFERENCES auth_users ON UPDATE CASCADE ON DELETE CASCADE,
    date           date NOT NULL,
    checkin        time NOT NULL,
    checkin_photo  bytea,
    checkout       time,
    checkout_photo bytea,
    workspace_id   serial REFERENCES workspaces ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE workspace_invitations (
    id           serial PRIMARY KEY,
    user_email   varchar NOT NULL REFERENCES auth_users(email) ON UPDATE CASCADE ON DELETE CASCADE,
    token        varchar NOT NULL,
    created_at   timestamp DEFAULT now() NOT NULL,
    expires_at   timestamp NOT NULL,
    confirmed_at timestamp,
    workspace_id serial NOT NULL REFERENCES workspaces ON UPDATE CASCADE ON DELETE CASCADE
);

-- Set ownership for all tables
ALTER TABLE roles OWNER TO postgres;
ALTER TABLE auth_users OWNER TO postgres;
ALTER TABLE confirmation_tokens OWNER TO postgres;
ALTER TABLE password_reset_tokens OWNER TO postgres;
ALTER TABLE importance OWNER TO postgres;
ALTER TABLE status OWNER TO postgres;
ALTER TABLE ev_subscriptions OWNER TO postgres;
ALTER TABLE countries OWNER TO postgres;
ALTER TABLE phone_dial_codes OWNER TO postgres;
ALTER TABLE workspace_roles OWNER TO postgres;
ALTER TABLE user_roles OWNER TO postgres;
ALTER TABLE workspaces OWNER TO postgres;
ALTER TABLE tasks_category OWNER TO postgres;
ALTER TABLE tasks OWNER TO postgres;
ALTER TABLE problems OWNER TO postgres;
ALTER TABLE worker_workspace_data OWNER TO postgres;
ALTER TABLE positions OWNER TO postgres;
ALTER TABLE full_users OWNER TO postgres;
ALTER TABLE workspace_users OWNER TO postgres;
ALTER TABLE conversations OWNER TO postgres;
ALTER TABLE messages OWNER TO postgres;
ALTER TABLE conversation_participants OWNER TO postgres;
ALTER TABLE users_citizenships OWNER TO postgres;
ALTER TABLE attendance OWNER TO postgres;
ALTER TABLE workspace_invitations OWNER TO postgres;
