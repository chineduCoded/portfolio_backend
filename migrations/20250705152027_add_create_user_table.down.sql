-- Down migration script
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
DROP FUNCTION IF EXISTS update_modified_column();
DROP INDEX IF EXISTS idx_users_email_lower;
DROP INDEX IF EXISTS idx_users_deleted_at;
DROP INDEX IF EXISTS idx_users_deleted_by;
DROP INDEX IF EXISTS idx_users_admin;
DROP INDEX IF EXISTS idx_users_verified;
DROP INDEX IF EXISTS idx_users_purge_eligible;
DROP FUNCTION IF EXISTS should_purge_user;
DROP TABLE IF EXISTS users CASCADE;
