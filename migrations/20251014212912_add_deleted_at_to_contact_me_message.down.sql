-- Remove the deleted_at column and related indexes
DROP INDEX IF EXISTS idx_contact_me_messages_deleted_at;
DROP INDEX IF EXISTS idx_contact_me_messages_active;

ALTER TABLE contact_me_messages 
DROP COLUMN deleted_at;