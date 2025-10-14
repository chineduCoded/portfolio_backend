-- Add up migration script here

-- Required extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Contact Me table
CREATE TABLE contact_me_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    subject TEXT,
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_contact_me_message_email ON contact_me_messages (email);
