-- Add up migration script here

-- Required extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Blog Posts table
-- This table stores blog posts with metadata such as title, slug, excerpt, content,
-- cover image URL, tags, SEO title, and description.
-- It includes fields for publication status, timestamps for creation and updates,
-- and supports soft deletion by including a deleted_at timestamp.

CREATE TABLE blog_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    excerpt TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    cover_image_url TEXT,
    tags TEXT[],
    seo_title TEXT,
    seo_description TEXT,
    published BOOLEAN NOT NULL DEFAULT FALSE,
    published_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ DEFAULT NULL
);

-- Indexes for performance optimization
-- These indexes improve query performance for common operations such as searching by slug,
-- filtering by published status, and sorting by publication date.
CREATE UNIQUE INDEX blog_posts_slug_idx ON blog_posts (slug);
CREATE INDEX blog_posts_published_at_idx ON blog_posts (published_at DESC);
CREATE INDEX blog_posts_published_idx ON blog_posts (published) WHERE published = true;
CREATE INDEX blog_posts_tags_idx ON blog_posts USING GIN (tags);
