-- Add up migration script here

-- Required extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- About Me table
-- This table stores the "About Me" content with revisions and effective dates
-- It allows for versioning of content, enabling users to retrieve historical versions
-- and manage content updates effectively
-- The table includes fields for content in Markdown format, effective date, and revision number
-- It also supports soft deletion by including a deleted_at timestamp

CREATE TABLE about_me (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    revision INT NOT NULL CHECK (revision >= 0),
    content_markdown TEXT NOT NULL,
    effective_date DATE NOT NULL CHECK (effective_date >= '1900-01-01'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ DEFAULT NULL,
    UNIQUE (effective_date, revision)
);

-- Create index for current content
CREATE INDEX idx_about_me_current 
ON about_me (effective_date DESC, revision DESC);

-- Create index for revisions
CREATE INDEX idx_about_me_revisions 
ON about_me (effective_date, revision DESC);

-- Partial index for filtering out deleted records
CREATE INDEX idx_about_me_active 
ON about_me (effective_date DESC, revision DESC) 
WHERE deleted_at IS NULL;