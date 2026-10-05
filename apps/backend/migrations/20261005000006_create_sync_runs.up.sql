-- How far each sync has got. A row is written as the sync starts and filled in
-- as it ends, so `/api/status` can report on a sync that is still running, as
-- well as on the last one that finished. Only the most recent run of each target
-- is kept.
CREATE TABLE sync_runs (
    id serial PRIMARY KEY,
    -- Which dataset was synced: premises, jobs or postcodes.
    target text NOT NULL,
    -- What it read: the URL of a dataset, or the path of a local file.
    source text NOT NULL,
    state text NOT NULL CHECK (
        state IN ('running', 'synced', 'unchanged', 'failed')
    ),
    started_at timestamp with time zone NOT NULL DEFAULT now(),
    -- Left null while the sync is still running.
    finished_at timestamp with time zone,
    -- Rows the table holds once the sync is done.
    rows bigint,
    -- Why the sync did what it did, and why it failed when it failed.
    message text,
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    updated_at timestamp with time zone NOT NULL DEFAULT now()
);

CREATE INDEX sync_runs_target_idx ON sync_runs (target);

CREATE TRIGGER sync_runs_set_updated_at
    BEFORE UPDATE ON sync_runs
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();