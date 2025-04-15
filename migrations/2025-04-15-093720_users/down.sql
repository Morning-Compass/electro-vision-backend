DROP TABLE IF EXISTS users_citizenships;

DROP TABLE IF EXISTS conversation_participants;

DROP TABLE IF EXISTS messages;

DROP TABLE IF EXISTS conversations;

DROP TABLE IF EXISTS workspace_users;

DROP TABLE IF EXISTS full_users;

DROP TABLE IF EXISTS countries;

DROP TABLE IF EXISTS phone_dial_codes;

DROP TABLE IF EXISTS positions;

DROP TABLE IF EXISTS worker_workspace_data;

DROP TABLE IF EXISTS problems;

DROP TABLE IF EXISTS tasks_category;

DROP TABLE IF EXISTS tasks;

DROP TABLE IF EXISTS workspaces;

DROP TABLE IF EXISTS ev_subscriptions;

DROP TABLE IF EXISTS workspace_roles;

DROP TABLE IF EXISTS status;

DROP TABLE IF EXISTS importance;

DROP TABLE IF EXISTS user_roles;

DROP TABLE IF EXISTS confirmation_tokens;

DROP TABLE IF EXISTS password_reset_tokens;

DROP TABLE IF EXISTS auth_users;

DROP TABLE IF EXISTS roles;

DROP TABLE IF EXISTS __diesel_schema_migrations;

-- Drop sequences
DROP SEQUENCE IF EXISTS users_id_seq;

DROP SEQUENCE IF EXISTS workspace_id_seq;

DROP SEQUENCE IF EXISTS workspace_owner_id_seq;

DROP SEQUENCE IF EXISTS tasks_worker_id_seq;

DROP SEQUENCE IF EXISTS phone_dial_codes_country_seq;

-- Drop functions
DROP FUNCTION IF EXISTS init_status_values ();

DROP FUNCTION IF EXISTS diesel_set_updated_at ();

DROP FUNCTION IF EXISTS diesel_manage_updated_at (_tbl regclass);
