//! How far each sync has got, recorded in the `sync_runs` table.
//!
//! The Next.js worker held this in memory, in the `status` object
//! `apps/worker/index.ts` answered `/status` with, which went with the process.
//! A sync here is a `sync` command run by cron instead, so it writes where it
//! has got to as it goes: `/api/status` can then report a sync that is still
//! running, and a run that was killed part way through leaves a trace saying so
//! rather than nothing at all.
//!
//! Only the most recent run of each target is kept, which is all the status page
//! has ever shown.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{AssertSqlSafe, FromRow, PgPool};
use tracing::warn;

use crate::search::{serialize_optional_timestamp, serialize_timestamp};

/// The dataset [`sync`](crate::sync::premises) reads a CSV for.
pub const PREMISES: &str = "premises";

/// The collections [`sync`](crate::sync::jobs) reads a CSV for.
pub const JOBS: &str = "jobs";

/// Why an unchanged upstream file says so itself, so the status page has
/// something to read rather than a blank.
const UNCHANGED: &str = "upstream has not changed, so nothing was imported";

/// How far a sync got.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// Started, and not finished yet.
    Running,
    /// The data was replaced with what upstream publishes.
    Synced,
    /// Upstream has not changed since the last sync, so nothing was imported.
    Unchanged,
    /// Stopped part way through, with the reason why in `message`.
    Failed,
}

impl State {
    fn as_str(self) -> &'static str {
        match self {
            State::Running => "running",
            State::Synced => "synced",
            State::Unchanged => "unchanged",
            State::Failed => "failed",
        }
    }

    /// The state a stored name is, or `None` for a name this build does not know.
    fn parse(raw: &str) -> Option<State> {
        [
            State::Running,
            State::Synced,
            State::Unchanged,
            State::Failed,
        ]
        .into_iter()
        .find(|state| state.as_str() == raw)
    }
}

/// What a sync ended up doing. Written as [`finish`] is called, whether the sync
/// worked or not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The import ran, and the tables now hold what upstream publishes.
    Synced { rows: i64 },
    /// Upstream has not changed since the last sync, so nothing was imported.
    Unchanged,
    /// The sync stopped part way through, with the reason why.
    Failed(String),
}

/// The most recent sync of a target, as `/api/status` serves it.
///
/// `id`, `target` and `source` are not served: the row is found by its dataset,
/// and the source is the dataset's own URL unless it was read from a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    #[serde(skip_serializing)]
    pub id: i32,
    #[serde(skip_serializing)]
    pub target: String,
    #[serde(skip_serializing)]
    pub source: String,
    pub state: State,
    #[serde(serialize_with = "serialize_timestamp")]
    pub started_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_optional_timestamp")]
    pub finished_at: Option<DateTime<Utc>>,
    /// Rows the table holds now the sync is done.
    pub rows: Option<i64>,
    /// What the sync did, or why it failed.
    pub message: Option<String>,
}

/// A stored run. `state` is read as text rather than as an enum, because the
/// column is a plain text one with a check constraint on it.
#[derive(Debug, Clone, FromRow)]
struct Row {
    id: i32,
    target: String,
    source: String,
    state: String,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    rows: Option<i64>,
    message: Option<String>,
}

impl Row {
    /// A run as it is served, where a state this build does not know is reported
    /// as a failure rather than left out: the column only allows the four states
    /// above, so anything else was written by something else, and the status
    /// page is more use saying so than answering nothing.
    fn into_run(self) -> Run {
        let state = State::parse(&self.state).unwrap_or_else(|| {
            warn!(state = %self.state, "unknown sync state");
            State::Failed
        });

        let message = match (state, self.message) {
            (State::Failed, None) => Some(format!("unknown sync state {:?}", self.state)),
            (_, message) => message,
        };

        Run {
            id: self.id,
            target: self.target,
            source: self.source,
            state,
            started_at: self.started_at,
            finished_at: self.finished_at,
            rows: self.rows,
            message,
        }
    }
}

const COLUMNS: &str = "id, target, source, state, started_at, finished_at, rows, message";

/// Records that a sync of `target` has started reading `source`, which is a URL
/// or the path of a local file.
pub async fn start(pool: &PgPool, target: &str, source: &str) -> Result<Run> {
    let sql = AssertSqlSafe(format!(
        "INSERT INTO sync_runs (target, source, state) VALUES ($1, $2, 'running') \
         RETURNING {COLUMNS}"
    ));

    let run: Row = sqlx::query_as(sql)
        .bind(target)
        .bind(source)
        .fetch_one(pool)
        .await
        .with_context(|| format!("recording the start of a {target} sync"))?;

    // The run this one replaces is no longer of any use, and there is one of
    // these per sync, so the table is kept to a row per target.
    sqlx::query("DELETE FROM sync_runs WHERE target = $1 AND id <> $2")
        .bind(target)
        .bind(run.id)
        .execute(pool)
        .await
        .with_context(|| format!("clearing the previous {target} sync"))?;

    Ok(run.into_run())
}

/// Records how a sync ended, which is the last thing it does: a sync that stops
/// early still has to be reported as having stopped.
pub async fn finish(pool: &PgPool, run: &Run, outcome: Outcome) -> Result<()> {
    let (state, rows, message): (State, Option<i64>, Option<&str>) = match &outcome {
        Outcome::Synced { rows } => (State::Synced, Some(*rows), None),
        Outcome::Unchanged => (State::Unchanged, None, Some(UNCHANGED)),
        Outcome::Failed(message) => (State::Failed, None, Some(message)),
    };

    sqlx::query(
        "UPDATE sync_runs SET state = $2, finished_at = now(), rows = $3, message = $4 \
         WHERE id = $1",
    )
    .bind(run.id)
    .bind(state.as_str())
    .bind(rows)
    .bind(message)
    .execute(pool)
    .await
    .with_context(|| format!("recording how the {target} sync ended", target = run.target))?;

    Ok(())
}

/// The most recent run of each of `targets`, where a target that has never been
/// synced is missing rather than blank.
pub async fn latest(pool: &PgPool, targets: &[&str]) -> Result<Vec<Run>> {
    if targets.is_empty() {
        return Ok(Vec::new());
    }

    let sql = AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM sync_runs WHERE target = ANY($1)"
    ));

    let rows: Vec<Row> = sqlx::query_as(sql)
        .bind(targets)
        .fetch_all(pool)
        .await
        .with_context(|| format!("reading the sync runs of {} targets", targets.len()))?;

    Ok(rows.into_iter().map(Row::into_run).collect())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use serde_json::json;

    use super::*;

    fn row(state: &str, message: Option<&str>) -> Row {
        Row {
            id: 7,
            target: PREMISES.to_owned(),
            source: "https://example.com/dm_premises.csv".to_owned(),
            state: state.to_owned(),
            started_at: Utc.with_ymd_and_hms(2026, 10, 5, 4, 0, 0).unwrap(),
            finished_at: Some(Utc.with_ymd_and_hms(2026, 10, 5, 4, 1, 2).unwrap()),
            rows: Some(411_468),
            message: message.map(str::to_owned),
        }
    }

    #[test]
    fn a_run_is_served_with_the_fields_the_status_page_reads() {
        let json = serde_json::to_value(row("synced", None).into_run()).expect("serialising");

        assert_eq!(
            json,
            json!({
                "state": "synced",
                "startedAt": "2026-10-05T04:00:00.000Z",
                "finishedAt": "2026-10-05T04:01:02.000Z",
                "rows": 411_468,
                "message": null,
            }),
            "only the fields the page reads are served"
        );
    }

    #[test]
    fn a_run_that_is_still_going_has_no_end() {
        let mut row = row("running", None);
        row.finished_at = None;
        row.rows = None;

        let json = serde_json::to_value(row.into_run()).expect("serialising");

        assert_eq!(
            json,
            json!({
                "state": "running",
                "startedAt": "2026-10-05T04:00:00.000Z",
                "finishedAt": null,
                "rows": null,
                "message": null,
            })
        );
    }

    #[test]
    fn every_state_reads_back_as_it_was_written() {
        for state in ["running", "synced", "unchanged", "failed"] {
            assert_eq!(
                State::parse(state).map(|state| state.as_str()),
                Some(state),
                "{state}"
            );
        }

        assert_eq!(State::parse(""), None);
        assert_eq!(State::parse("Synced"), None);
    }

    #[test]
    fn a_state_this_build_does_not_know_is_reported_as_a_failure() {
        let run = row("halfway", None).into_run();

        assert_eq!(run.state, State::Failed);
        assert_eq!(
            run.message.as_deref(),
            Some("unknown sync state \"halfway\""),
            "the reason says what was read, rather than leaving a blank"
        );
    }

    #[test]
    fn a_failure_keeps_the_reason_it_was_given() {
        let run = row("failed", Some("could not reach upstream")).into_run();

        assert_eq!(run.state, State::Failed);
        assert_eq!(run.message.as_deref(), Some("could not reach upstream"));
    }
}
