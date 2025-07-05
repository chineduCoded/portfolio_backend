CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(100),
    password_hash TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),  
    deleted_at TIMESTAMPTZ,                         
    deleted_by UUID REFERENCES users(id)
);

-- Case-insensitive email index
CREATE UNIQUE INDEX idx_users_email_lower ON users (LOWER(email));

-- Indexes for soft-delete system
CREATE INDEX idx_users_deleted_at ON users (deleted_at);
CREATE INDEX idx_users_deleted_by ON users (deleted_by); 

-- Partial indexes for admin and verified users
CREATE INDEX idx_users_admin ON users (id) WHERE is_admin = true;
CREATE INDEX idx_users_verified ON users (id) WHERE is_verified = true;

-- Create purge eligibility function FIRST
CREATE OR REPLACE FUNCTION should_purge_user(deleted_at TIMESTAMPTZ)
RETURNS BOOLEAN IMMUTABLE AS $$
    SELECT deleted_at < (NOW() - INTERVAL '7 days');
$$ LANGUAGE SQL;

-- Create index that uses the function
CREATE INDEX idx_users_purge_eligible ON users (id) 
WHERE deleted_at IS NOT NULL AND should_purge_user(deleted_at);

-- Create trigger function for automatic updated_at
CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply trigger to users table
CREATE TRIGGER update_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();

-- Add comments for documentation
COMMENT ON TABLE users IS 'Stores user account information';
COMMENT ON COLUMN users.id IS 'Unique user identifier (UUID)';
COMMENT ON COLUMN users.email IS 'User email address (case-insensitive unique)';
COMMENT ON COLUMN users.username IS 'Optional display name';
COMMENT ON COLUMN users.password_hash IS 'Bcrypt password hash';
COMMENT ON COLUMN users.is_admin IS 'Administrator flag';
COMMENT ON COLUMN users.is_verified IS 'Email verification status';
COMMENT ON COLUMN users.created_at IS 'Timestamp of account creation';
COMMENT ON COLUMN users.updated_at IS 'Timestamp of last update (auto-updated)';
COMMENT ON COLUMN users.deleted_at IS 'Soft delete timestamp (NULL if active)';
COMMENT ON COLUMN users.deleted_by IS 'User ID of who performed deletion';

-- Add Foreign Key Constraint
ALTER TABLE users
ADD CONSTRAINT fk_deleted_by 
FOREIGN KEY (deleted_by) REFERENCES users(id)
ON DELETE SET NULL;

-- Add Audit Table
CREATE TABLE user_audit (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    action TEXT NOT NULL,
    performed_by UUID REFERENCES users(id),
    performed_at TIMESTAMPTZ DEFAULT NOW()
);