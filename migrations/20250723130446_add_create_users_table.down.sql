-- Add down migration script here
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
DROP FUNCTION IF EXISTS update_modified_column();
DROP FUNCTION IF EXISTS should_purge_user;
DROP INDEX IF EXISTS unique_active_users_email;
DROP INDEX IF EXISTS idx_users_deleted_at;
DROP INDEX IF EXISTS idx_users_deleted_by;
DROP INDEX IF EXISTS idx_users_admin;
DROP INDEX IF EXISTS idx_users_verified;
DROP INDEX IF EXISTS idx_users_purge_eligible;
DROP TABLE IF EXISTS user_audit;
DROP TABLE IF EXISTS users CASCADE;
