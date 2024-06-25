-- Table for "users"
CREATE TABLE "users" (
    "id" SERIAL PRIMARY KEY,
    "username" VARCHAR NOT NULL,
    "email" VARCHAR NOT NULL UNIQUE,
    "password" VARCHAR NOT NULL,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "account_valid" BOOL NOT NULL DEFAULT FALSE
);

-- Table for "confirmation_tokens"
CREATE TABLE "confirmation_tokens" (
    "id" SERIAL PRIMARY KEY,
    "user_email" VARCHAR NOT NULL,
    "token" VARCHAR NOT NULL,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "expires_at" TIMESTAMP NOT NULL,
    "confirmed_at" TIMESTAMP,
    FOREIGN KEY ("user_email") REFERENCES "users"("email")
);

-- Table for "roles"
CREATE TABLE "roles" (
    "id" SERIAL PRIMARY KEY,
    "name" VARCHAR NOT NULL UNIQUE
);

-- Table for "user_roles"
CREATE TABLE "user_roles" (
    "user_id" INTEGER NOT NULL,
    "role_id" INTEGER NOT NULL,
    PRIMARY KEY ("user_id", "role_id"),
    FOREIGN KEY ("user_id") REFERENCES "users"("id") ON DELETE CASCADE,
    FOREIGN KEY ("role_id") REFERENCES "roles"("id") ON DELETE CASCADE
);
