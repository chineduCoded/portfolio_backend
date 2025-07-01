-- Add down migration script here
DROP TABLE IF EXISTS users;
DROP INDEX IF EXISTS idx_users_email;
