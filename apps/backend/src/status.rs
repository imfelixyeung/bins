//! What `GET /api/status` reports: whether each dataset has changed upstream,
//! when it was last looked at, when its data was last replaced, and how far the
//! last sync of it got.
//!
//! The status page used to read the `etags` table through drizzle, and the
//! Next.js worker kept its own progress in memory to answer `/status`. Both now
//! live in the database, so one endpoint can serve the lot: the datasets come
//! from the same constants the sync commands default to, and the progress from
//! the `sync_runs` rows the syncs write.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::etag::{self, Stored};
use crate::import::{jobs, premises};
use crate::search::serialize_optional_timestamp;
use crate::sync_run::{self, Run};

/// The datasets `/api/status` reports on, in the order they are listed.
///
/// The URLs are the defaults the sync commands fall back on, so a dataset is
/// named here rather than being listed by the page, and the two cannot drift
/// apart.
const DATASETS: [(&str, &str, &str); 2] = [
    ("jobs", "Jobs", jobs::DEFAULT_URL),
    ("premises", "Premises", premises::DEFAULT_URL),
];

/// Everything `/api/status` serves.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub datasets: Vec<Dataset>,
}

/// One dataset, as the status page lists it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    /// The target its sync is recorded under.
    pub key: &'static str,
    pub name: &'static str,
    pub url: &'static str,
    /// The etag upstream sent the last time it was asked about the file, which
    /// is `null` until the file has been imported once.
    pub etag: Option<String>,
    /// When upstream was last asked, whether or not anything had changed since
    /// the run before.
    #[serde(serialize_with = "serialize_optional_timestamp")]
    pub last_checked: Option<DateTime<Utc>>,
    /// When the file last changed upstream, as far as the data imported from it
    /// goes: this is the `last-modified` header of the version that was
    /// imported, which is what the status page called "last updated".
    #[serde(serialize_with = "serialize_optional_timestamp")]
    pub last_synced: Option<DateTime<Utc>>,
    /// The last sync of this dataset, which is absent until one has run.
    pub sync: Option<Run>,
}

/// Reads the status of every dataset.
pub async fn read(pool: &PgPool) -> Result<Status> {
    let targets: Vec<&str> = DATASETS.iter().map(|&(key, _, _)| key).collect::<Vec<_>>();

    // Two queries rather than one join, since a dataset can have an etag without
    // having been synced yet, and a run without an etag when the check failed.
    let runs = sync_run::latest(pool, &targets).await?;
    let mut datasets = Vec::with_capacity(DATASETS.len());

    for &(key, name, url) in &DATASETS {
        let stored: Option<Stored> = etag::stored(pool, url).await?;

        datasets.push(Dataset {
            key,
            name,
            url,
            etag: stored.as_ref().and_then(|stored| stored.etag.clone()),
            last_checked: stored.as_ref().map(|stored| stored.checked_at),
            last_synced: stored.as_ref().and_then(|stored| stored.modified_at),
            sync: runs.iter().find(|run| run.target == key).cloned(),
        });
    }

    Ok(Status { datasets })
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use serde_json::json;

    use super::*;
    use crate::sync_run::State;

    fn when() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 5, 4, 0, 0).unwrap()
    }

    fn dataset() -> Dataset {
        Dataset {
            key: "premises",
            name: "Premises",
            url: premises::DEFAULT_URL,
            etag: Some("\"f01141fe7b54dd1:0\"".to_owned()),
            last_checked: Some(when()),
            last_synced: Some(when()),
            sync: Some(Run {
                id: 1,
                target: "premises".to_owned(),
                source: premises::DEFAULT_URL.to_owned(),
                state: State::Synced,
                started_at: when(),
                finished_at: Some(when()),
                rows: Some(411_468),
                message: None,
            }),
        }
    }

    #[test]
    fn a_dataset_is_served_with_what_the_status_page_reads() {
        let json = serde_json::to_value(dataset()).expect("serialising");

        assert_eq!(
            json,
            json!({
                "key": "premises",
                "name": "Premises",
                "url": premises::DEFAULT_URL,
                "etag": "\"f01141fe7b54dd1:0\"",
                "lastChecked": "2026-10-05T04:00:00.000Z",
                "lastSynced": "2026-10-05T04:00:00.000Z",
                "sync": {
                    "state": "synced",
                    "startedAt": "2026-10-05T04:00:00.000Z",
                    "finishedAt": "2026-10-05T04:00:00.000Z",
                    "rows": 411468,
                    "message": null,
                },
            })
        );
    }

    #[test]
    fn a_dataset_that_has_never_been_synced_is_null_rather_than_missing() {
        let mut dataset = dataset();
        dataset.etag = None;
        dataset.last_checked = None;
        dataset.last_synced = None;
        dataset.sync = None;

        let json = serde_json::to_value(dataset).expect("serialising");

        // The page reads these fields directly, so they are answered with null
        // rather than left out of the response.
        assert_eq!(json["etag"], json!(null));
        assert_eq!(json["lastChecked"], json!(null));
        assert_eq!(json["lastSynced"], json!(null));
        assert_eq!(json["sync"], json!(null));
        assert_eq!(
            json.as_object().expect("an object").len(),
            7,
            "the fields are served whether or not they hold anything"
        );
    }

    #[test]
    fn the_datasets_listed_are_the_ones_the_syncs_default_to() {
        let listed: Vec<(&str, &str)> = DATASETS.iter().map(|&(key, _, url)| (key, url)).collect();

        assert_eq!(
            listed,
            vec![
                (sync_run::JOBS, jobs::DEFAULT_URL),
                (sync_run::PREMISES, premises::DEFAULT_URL),
            ]
        );
    }
}
