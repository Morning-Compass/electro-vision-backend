DROP TABLE IF EXISTS user_roles;
DROP TABLE IF EXISTS roles;
DROP TABLE IF EXISTS confirmation_tokens;
DROP TABLE IF EXISTS users;


INSERT INTO users (username, email, password, created_at, account_valid) VALUES
('john_doe', 'john.doe@example.com', 'password123', '2024-06-01 10:00:00', TRUE),
('jane_smith', 'jane.smith@example.com', 'securepassword', '2024-06-02 11:00:00', FALSE),
('alice_jones', 'alice.jones@example.com', 'mypassword', '2024-06-03 12:00:00', TRUE);

INSERT INTO confirmation_tokens (user_email, token, created_at, expires_at, confirmed_at) VALUES
('john.doe@example.com', 'token123', '2024-06-01 10:05:00', '2024-06-01 12:05:00', '2024-06-01 11:00:00'),
('jane.smith@example.com', 'token456', '2024-06-02 11:10:00', '2024-06-02 13:10:00', NULL),
('alice.jones@example.com', 'token789', '2024-06-03 12:15:00', '2024-06-03 14:15:00', '2024-06-03 13:00:00');

INSERT INTO roles (name) VALUES
('Admin'),
('User'),
('Guest');

INSERT INTO user_roles (user_id, role_id) VALUES
(1, 1), -- John Doe as Admin
(2, 2), -- Jane Smith as User
(3, 3); -- Alice Jones as Guest

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    account_valid BOOLEAN NOT NULL
);

CREATE TABLE confirmation_tokens (
    id SERIAL PRIMARY KEY,
    user_email VARCHAR NOT NULL,
    token VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    confirmed_at TIMESTAMP
);

CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL
);

CREATE TABLE user_roles (
    user_id INT NOT NULL,
    role_id INT NOT NULL,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (role_id) REFERENCES roles(id)
);
