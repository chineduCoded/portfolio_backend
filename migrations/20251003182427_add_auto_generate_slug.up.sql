-- Drop old unique index / constraint on slug
DROP INDEX IF EXISTS blog_posts_slug_idx;
ALTER TABLE blog_posts DROP CONSTRAINT IF EXISTS blog_posts_slug_key;

-- Ensure slug is NOT NULL
ALTER TABLE blog_posts
  ALTER COLUMN slug SET NOT NULL;

-- Create partial unique index to enforce case-insensitive uniqueness
-- only on active (non-deleted) posts
CREATE UNIQUE INDEX IF NOT EXISTS blog_posts_slug_active_idx
  ON blog_posts (LOWER(slug))
  WHERE deleted_at IS NULL;
