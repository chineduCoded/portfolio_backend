-- Add columns
ALTER TABLE users
    ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN is_verified BOOLEAN NOT NULL DEFAULT false;

-- Create partial index for admin users (optional but recommended)
CREATE INDEX idx_users_admin ON users (id) WHERE is_admin = true;

-- Update existing users if needed (example: mark one admin)
-- UPDATE users SET is_admin = true WHERE email = 'admin@example.com';
