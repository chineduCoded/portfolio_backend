-- Add down migration script here

-- Remove trigger before dropping function
DROP TRIGGER IF EXISTS about_me_revision_trigger ON about_me;

-- Remove function
DROP FUNCTION IF EXISTS set_about_me_revision();

