-- Add up migration script here
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

CREATE TRIGGER about_me_revision_trigger
BEFORE INSERT ON about_me
FOR EACH ROW
EXECUTE FUNCTION set_about_me_revision();

