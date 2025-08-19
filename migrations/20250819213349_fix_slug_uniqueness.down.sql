-- Add down migration script here

-- Drop the partial unique index we added
DROP INDEX IF EXISTS blog_posts_slug_active_idx;

-- Restore the previous state by dropping the partial unique index
-- and ensuring the slug column is unique again
-- This will restore the previous uniqueness constraint on the slug column
-- which was case-sensitive and applied to all posts
-- This is necessary to revert the changes made in the up migration
ALTER TABLE blog_posts
  ALTER COLUMN slug DROP NOT NULL;
-- Drop the unique constraint that was added in the up migration
ALTER TABLE blog_posts
  DROP CONSTRAINT IF EXISTS blog_posts_slug_active_key; 

-- Restore old uniqueness: add column-level unique constraint
ALTER TABLE blog_posts
  ADD CONSTRAINT blog_posts_slug_key UNIQUE (slug);

