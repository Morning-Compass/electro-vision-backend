-- Your SQL goes here
CREATE TABLE "confirmation_tokens"(
	"id" INTEGER NOT NULL PRIMARY KEY,
	"user_email" VARCHAR NOT NULL,
	"token" VARCHAR NOT NULL,
	"created_at" TIMESTAMP NOT NULL,
	"expires_at" TIMESTAMP NOT NULL,
	"confirmed_at" TIMESTAMP NOT NULL,
	FOREIGN KEY ("user_email") REFERENCES "user"("id")
);

CREATE TABLE "roles"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"name" VARCHAR NOT NULL
);

CREATE TABLE "user"(
	"id" INTEGER NOT NULL,
	"username" VARCHAR NOT NULL,
	"email" VARCHAR NOT NULL PRIMARY KEY,
	"password" VARCHAR NOT NULL,
	"created_at" TIMESTAMP NOT NULL,
	"account_valid" BOOL NOT NULL
);

CREATE TABLE "user_roles"(
	"user_id" INTEGER NOT NULL PRIMARY KEY,
	"role_id" INT4 NOT NULL
);

