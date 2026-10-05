-- What upstream says about each dataset it publishes, so a sync can tell
-- whether the file it is about to download is the one it already holds.
CREATE TABLE etags (
    id serial PRIMARY KEY,
    url text NOT NULL UNIQUE,
    etag text,
    -- When the file itself last changed, as sent by its `last-modified` header.
    -- Only written once the data has been imported, so this is the version of
    -- the file the tables hold.
    modified_at timestamp with time zone,
    -- When upstream was last asked about the file, whether or not anything had
    -- changed since the run before.
    checked_at timestamp with time zone NOT NULL DEFAULT now(),
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    updated_at timestamp with time zone NOT NULL DEFAULT now()
);

CREATE TRIGGER etags_set_updated_at
    BEFORE UPDATE ON etags
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();