-- Add down migration script here
-- This script is used to revert the changes made by the up migration script.
-- It drops the blog_posts table created in the up migration.
-- It is important to ensure that this script is executed in the correct order
-- to maintain database integrity and avoid errors.

DROP INDEX IF EXISTS blog_posts_tags_idx;
DROP INDEX IF EXISTS blog_posts_published_idx;
DROP INDEX IF EXISTS blog_posts_published_at_idx;
DROP INDEX IF EXISTS blog_posts_slug_idx;

DROP TABLE IF EXISTS blog_posts;