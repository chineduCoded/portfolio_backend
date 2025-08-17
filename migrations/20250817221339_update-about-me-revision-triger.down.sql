-- Drop the new trigger + function with advisory lock
DROP TRIGGER IF EXISTS about_me_revision_trigger ON about_me;
DROP FUNCTION IF EXISTS set_about_me_revision();

-- Recreate the old revision trigger function without advisory lock
CREATE OR REPLACE FUNCTION set_about_me_revision()
RETURNS TRIGGER AS $$
BEGIN
    -- Calculate next revision for this effective_date
    NEW.revision := COALESCE((
        SELECT MAX(revision) + 1
        FROM about_me
        WHERE effective_date = NEW.effective_date
          AND deleted_at IS NULL
    ), 1);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Recreate trigger
CREATE TRIGGER about_me_revision_trigger
BEFORE INSERT ON about_me
FOR EACH ROW
EXECUTE FUNCTION set_about_me_revision();
