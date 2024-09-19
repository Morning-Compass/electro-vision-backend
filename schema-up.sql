CREATE TABLE confirmation_tokens (
    id SERIAL PRIMARY KEY,
    user_email VARCHAR NOT NULL,
    token VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    confirmed_at TIMESTAMP NULL
);

CREATE TABLE roles (
    id SMALLINT PRIMARY KEY,
    name VARCHAR NOT NULL
);

CREATE TABLE user_roles (
    user_id INT NOT NULL,
    role_id SMALLINT NOT NULL,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    account_valid BOOLEAN NOT NULL
);

ALTER TABLE user_roles
ADD CONSTRAINT fk_user_roles_users FOREIGN KEY (user_id) REFERENCES users (id),
ADD CONSTRAINT fk_user_roles_roles FOREIGN KEY (role_id) REFERENCES roles (id);
