-- Add up migration script here

-- Required extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL,
    username VARCHAR(100),
    password_hash TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),  
    deleted_at TIMESTAMPTZ,
    deleted_by UUID
);

-- Foreign key constraint for deleted_by
ALTER TABLE users
ADD CONSTRAINT fk_deleted_by 
FOREIGN KEY (deleted_by) REFERENCES users(id)
ON DELETE SET NULL;

-- ✅ Partial unique index for ACTIVE emails (case-insensitive)
CREATE UNIQUE INDEX unique_active_users_email
ON users (LOWER(email))
WHERE deleted_at IS NULL;

-- Indexes for soft-delete system
CREATE INDEX idx_users_deleted_at ON users (deleted_at);
CREATE INDEX idx_users_deleted_by ON users (deleted_by);

-- Partial indexes for admin and verified users
CREATE INDEX idx_users_admin ON users (id) WHERE is_admin = true;
CREATE INDEX idx_users_verified ON users (id) WHERE is_verified = true;

-- Function to check purge eligibility
CREATE OR REPLACE FUNCTION should_purge_user(deleted_at TIMESTAMPTZ)
RETURNS BOOLEAN IMMUTABLE AS $$
    SELECT deleted_at < (NOW() - INTERVAL '7 days');
$$ LANGUAGE SQL;

-- Index to optimize purge queries
CREATE INDEX idx_users_purge_eligible ON users (id)
WHERE deleted_at IS NOT NULL AND should_purge_user(deleted_at);

-- Trigger for auto-updating updated_at
CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();

-- Audit table
CREATE TABLE user_audit (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    action TEXT NOT NULL,
    performed_by UUID REFERENCES users(id),
    performed_at TIMESTAMPTZ DEFAULT NOW()
);

-- Documentation comments
COMMENT ON TABLE users IS 'Stores user account information';
COMMENT ON COLUMN users.email IS 'User email (case-insensitive, unique among active)';
COMMENT ON COLUMN users.deleted_at IS 'Soft delete timestamp';
COMMENT ON COLUMN users.deleted_by IS 'ID of admin/user who deleted the account';
COMMENT ON FUNCTION should_purge_user IS 'Checks if a user is eligible for purge based on deletion date';
COMMENT ON FUNCTION update_modified_column IS 'Updates the updated_at timestamp on user updates';
COMMENT ON TABLE user_audit IS 'Stores audit logs for user actions';
