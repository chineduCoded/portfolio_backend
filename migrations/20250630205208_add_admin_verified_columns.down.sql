-- Remove indexes first
DROP INDEX IF EXISTS idx_users_admin;

-- Remove columns
ALTER TABLE users
    DROP COLUMN is_admin,
    DROP COLUMN is_verified;

