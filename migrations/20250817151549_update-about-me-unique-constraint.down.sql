-- Drop the partial unique index
DROP INDEX IF EXISTS idx_about_me_active;

-- Recreate the original non-unique index for active records
CREATE INDEX idx_about_me_active 
ON about_me (effective_date DESC, revision DESC)
WHERE deleted_at IS NULL;

-- Restore the original unique constraint
ALTER TABLE about_me 
ADD CONSTRAINT about_me_effective_date_revision_key UNIQUE (effective_date, revision);

