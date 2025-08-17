-- Drop old unique constraint on (effective_date, revision)
ALTER TABLE about_me DROP CONSTRAINT about_me_effective_date_revision_key;

-- Drop the old non-unique active index if it exists
DROP INDEX IF EXISTS idx_about_me_active;

-- Create a partial UNIQUE index for active records only
CREATE UNIQUE INDEX idx_about_me_active 
ON about_me (effective_date DESC, revision DESC)
WHERE deleted_at IS NULL;

