

-- Recreate the tables in the correct order

-- Create the confirmation_tokens table
CREATE TABLE confirmation_tokens (
    id SERIAL PRIMARY KEY,
    user_email VARCHAR NOT NULL,
    token VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    confirmed_at TIMESTAMP NULL
);

-- Create the roles table
CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL DEFAULT 'USER'  
);

-- Create the users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    account_valid BOOLEAN NOT NULL
);

-- Create the user_roles table (with foreign key references to users and roles)
CREATE TABLE user_roles (
    user_id INT NOT NULL,
    role_id SERIAL NOT NULL,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE
);

-- INSERT INTO users (username, email, password, created_at, account_valid) VALUES ('tester', 'tomek@el-jot.eu', 'qazxsw2.', CURRENT_TIMESTAMP), true);
-- Add the constraints to the user_roles table (this step is redundant as the constraints were added during table creation)
--ALTER TABLE user_roles
--ADD CONSTRAINT fk_user_roles_users FOREIGN KEY (user_id) REFERENCES users (id),
--ADD CONSTRAINT fk_user_roles_roles FOREIGN KEY (role_id) REFERENCES roles (id);
