-- Add deleted_at column for soft delete functionality
ALTER TABLE contact_me_messages 
ADD COLUMN deleted_at TIMESTAMPTZ NULL;

-- Create an index for faster queries filtering out soft-deleted records
CREATE INDEX idx_contact_me_messages_deleted_at ON contact_me_messages (deleted_at);

-- Create a partial index for active records only
CREATE INDEX idx_contact_me_messages_active ON contact_me_messages (created_at) WHERE deleted_at IS NULL;
