ALTER TABLE user_roles
DROP CONSTRAINT IF EXISTS user_roles_user_id_fkey;

ALTER TABLE user_roles
DROP CONSTRAINT IF EXISTS user_roles_role_id_fkey;

-- DROP TABLE IF EXISTS user_roles;
-- DROP TABLE IF EXISTS users;
-- DROP TABLE IF EXISTS roles;
-- DROP TABLE IF EXISTS confirmation_tokens;
-- DROP DATABASE IF EXISTS your_database_name;
-- Drop tables in reverse order
DROP TABLE IF EXISTS user_roles;

DROP TABLE IF EXISTS users;

DROP TABLE IF EXISTS confirmation_tokens;

DROP TABLE IF EXISTS roles;
