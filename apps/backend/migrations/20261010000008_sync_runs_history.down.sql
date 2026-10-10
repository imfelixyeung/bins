DROP INDEX IF EXISTS sync_runs_target_started_at_idx;

CREATE INDEX sync_runs_target_idx ON sync_runs (target);
