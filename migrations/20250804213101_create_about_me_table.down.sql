-- Drop indexes first (required before dropping the table)
DROP INDEX IF EXISTS idx_about_me_active;
DROP INDEX IF EXISTS idx_about_me_revisions;
DROP INDEX IF EXISTS idx_about_me_current;

-- Drop the table
DROP TABLE IF EXISTS about_me;
