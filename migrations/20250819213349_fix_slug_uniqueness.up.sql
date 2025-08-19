-- Add up migration script here

-- Drop existing redundant unique index if it exists
DROP INDEX IF EXISTS blog_posts_slug_idx;

-- If slug column was declared UNIQUE, drop that constraint
ALTER TABLE blog_posts
  DROP CONSTRAINT IF EXISTS blog_posts_slug_key;

-- Ensure slug is still required
ALTER TABLE blog_posts
  ALTER COLUMN slug SET NOT NULL;

-- Create partial unique index to enforce case-insensitive uniqueness
-- only on active (non-deleted) posts
CREATE UNIQUE INDEX blog_posts_slug_active_idx
  ON blog_posts (LOWER(slug))
  WHERE deleted_at IS NULL;
