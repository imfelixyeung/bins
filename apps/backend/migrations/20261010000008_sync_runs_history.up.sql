-- Every sync run is now kept rather than only the most recent one of each target,
-- so the dataset history pages can list them. The index is widened to cover the
-- order those pages read in, which makes the old target-only index redundant.
DROP INDEX IF EXISTS sync_runs_target_idx;

CREATE INDEX sync_runs_target_started_at_idx
    ON sync_runs (target, started_at DESC);
