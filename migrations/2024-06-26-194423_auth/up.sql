-- Drop the table for user_roles
DROP TABLE IF EXISTS user_roles;

-- Drop the table for roles
DROP TABLE IF EXISTS roles;

-- Drop the table for confirmation_tokens
DROP TABLE IF EXISTS confirmation_tokens;

-- Drop the table for users
DROP TABLE IF EXISTS users;

-- Create table for users
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    account_valid BOOLEAN NOT NULL
);

-- Create table for confirmation_tokens
CREATE TABLE confirmation_tokens (
    id SERIAL PRIMARY KEY,
    user_email VARCHAR NOT NULL,
    token VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    confirmed_at TIMESTAMP
);

-- Create table for roles
CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL
);

-- Create table for user_roles
CREATE TABLE user_roles (
    user_id INT NOT NULL,
    role_id INT NOT NULL,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (role_id) REFERENCES roles(id)
);
