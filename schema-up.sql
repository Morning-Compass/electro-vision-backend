-- Drop constraints only if they exist
DO $$ 
BEGIN
    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'user_roles_user_id_fkey') THEN
        ALTER TABLE user_roles DROP CONSTRAINT user_roles_user_id_fkey;
    END IF;
    
    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'user_roles_role_id_fkey') THEN
        ALTER TABLE user_roles DROP CONSTRAINT user_roles_role_id_fkey;
    END IF;
END $$;

-- Drop the tables if they exist
DROP TABLE IF EXISTS user_roles CASCADE;
DROP TABLE IF EXISTS users CASCADE;
DROP TABLE IF EXISTS roles CASCADE;
DROP TABLE IF EXISTS confirmation_tokens CASCADE;

CREATE TABLE roles (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL DEFAULT 'USER'
);
-- Recreate the tables in the correct order
CREATE TABLE confirmation_tokens (
    id SERIAL PRIMARY KEY,
    user_email VARCHAR NOT NULL,
    token VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    confirmed_at TIMESTAMP NULL
);


CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    password VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    account_valid BOOLEAN NOT NULL
);

CREATE TABLE user_roles (
    user_id INT NOT NULL,
    role_id INT NOT NULL,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);
