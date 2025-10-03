-- Drop the partial case-insensitive index
DROP INDEX IF EXISTS blog_posts_slug_active_idx;

-- Restore simple unique constraint on slug (case-sensitive, applies to all rows)
ALTER TABLE blog_posts
  ADD CONSTRAINT blog_posts_slug_key UNIQUE (slug);

-- Optionally, restore the old explicit unique index (redundant with constraint)
-- CREATE UNIQUE INDEX blog_posts_slug_idx ON blog_posts (slug);
