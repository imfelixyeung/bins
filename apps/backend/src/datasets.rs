//! What `GET /api/datasets` reports: whether each dataset has changed upstream,
//! when it was last looked at, when its data was last replaced, how far the last
//! sync of it got, and how big it is both upstream and in the local mirror.
//!
//! Everything the dataset page reports lives in the database, so one endpoint
//! can serve the lot: the datasets come from the same constants the sync
//! commands default to, and the progress from the `sync_runs` rows the syncs
//! write.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{AssertSqlSafe, PgPool};

use crate::etag::{self, Stored};
use crate::import::{jobs, premises};
use crate::search::serialize_optional_timestamp;
use crate::sync_run::Run;

/// The datasets `/api/datasets` reports on, in the order they are listed.
///
/// The URLs are the defaults the sync commands fall back on, so a dataset is
/// named here rather than being listed by the page, and the two cannot drift
/// apart. Each row also carries the table its imports fill, which is what the
/// local mirror's size is read from.
const DATASETS: [(&str, &str, &str, &str); 2] = [
    ("jobs", "Jobs", jobs::DEFAULT_URL, jobs::table()),
    (
        "premises",
        "Premises",
        premises::DEFAULT_URL,
        premises::table(),
    ),
];

/// Everything `/api/datasets` serves.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Datasets {
    pub datasets: Vec<Dataset>,
}

/// One dataset, as the dataset page lists it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    /// The target its syncs are recorded under, and the slug of its history
    /// page.
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
    /// imported.
    #[serde(serialize_with = "serialize_optional_timestamp")]
    pub last_synced: Option<DateTime<Utc>>,
    /// How big the file upstream publishes is, in bytes, from its
    /// `content-length` header. `null` until upstream has said.
    pub csv_size: Option<i64>,
    /// How big the local mirror of the dataset is, in bytes: the size of the
    /// table on disk, which `pg_relation_size` reports. `null` before the
    /// dataset has been imported.
    pub db_size: Option<i64>,
    /// The last sync of this dataset, which is absent until one has run.
    pub sync: Option<Run>,
}

/// Reads the datasets, with how big each one is upstream and in the database.
pub async fn read(pool: &PgPool) -> Result<Datasets> {
    let targets: Vec<&str> = DATASETS
        .iter()
        .map(|&(key, _, _, _)| key)
        .collect::<Vec<_>>();

    // Two queries rather than one join, since a dataset can have an etag without
    // having been synced yet, and a run without an etag when the check failed.
    let runs = crate::sync_run::latest(pool, &targets).await?;
    let mut datasets = Vec::with_capacity(DATASETS.len());

    for &(key, name, url, table) in &DATASETS {
        let stored: Option<Stored> = etag::stored(pool, url).await?;

        datasets.push(Dataset {
            key,
            name,
            url,
            etag: stored.as_ref().and_then(|stored| stored.etag.clone()),
            last_checked: stored.as_ref().map(|stored| stored.checked_at),
            last_synced: stored.as_ref().and_then(|stored| stored.modified_at),
            csv_size: stored.as_ref().and_then(|stored| stored.size),
            db_size: table_size(pool, table).await?,
            sync: runs.iter().find(|run| run.target == key).cloned(),
        });
    }

    Ok(Datasets { datasets })
}

/// The table a dataset's sync fills, quoted, as `pg_relation_size` wants it.
///
/// The queries here interpolate this name, so it comes from the RowSpec of the
/// import rather than from anywhere a client can reach.
pub fn table(key: &str) -> Option<&'static str> {
    DATASETS
        .iter()
        .find(|dataset| dataset.0 == key)
        .map(|dataset| dataset.3)
}

/// How big a table is on disk, in bytes, or `null` when the data has never been
/// imported and the table holds nothing.
async fn table_size(pool: &PgPool, table: &str) -> Result<Option<i64>> {
    let size: i64 = sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT COALESCE(pg_relation_size('{table}'), 0)"
    )))
    .fetch_one(pool)
    .await
    .with_context(|| format!("reading the size of {table}"))?;

    // `pg_relation_size` reports 0 for a table that has been imported and
    // emptied, which this project never ships, so treat 0 as "not yet loaded"
    // rather than as a size worth showing.
    Ok((size > 0).then_some(size))
}

/// Whether `key` is one of the datasets this build knows, which decides whether
/// a history page exists for it.
pub fn knows(key: &str) -> bool {
    DATASETS.iter().any(|&(k, _, _, _)| k == key)
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
            csv_size: Some(170_000_000),
            db_size: Some(86_210_000),
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
    fn a_dataset_is_served_with_what_the_dataset_page_reads() {
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
                "csvSize": 170000000,
                "dbSize": 86210000,
                "sync": {
                    "id": 1,
                    "source": premises::DEFAULT_URL,
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
        dataset.csv_size = None;
        dataset.db_size = None;
        dataset.sync = None;

        let json = serde_json::to_value(dataset).expect("serialising");

        // The page reads these fields directly, so they are answered with null
        // rather than left out of the response.
        assert_eq!(json["etag"], json!(null));
        assert_eq!(json["lastChecked"], json!(null));
        assert_eq!(json["lastSynced"], json!(null));
        assert_eq!(json["csvSize"], json!(null));
        assert_eq!(json["dbSize"], json!(null));
        assert_eq!(json["sync"], json!(null));
        assert_eq!(
            json.as_object().expect("an object").len(),
            9,
            "the fields are served whether or not they hold anything"
        );
    }

    #[test]
    fn the_datasets_listed_are_the_ones_the_syncs_default_to() {
        let listed: Vec<(&str, &str, &str)> = DATASETS
            .iter()
            .map(|&(key, _, url, table)| (key, url, table))
            .collect();

        assert_eq!(
            listed,
            vec![
                (crate::sync_run::JOBS, jobs::DEFAULT_URL, jobs::table(),),
                (
                    crate::sync_run::PREMISES,
                    premises::DEFAULT_URL,
                    premises::table(),
                ),
            ]
        );
    }

    #[test]
    fn a_key_is_known_only_when_its_dataset_is() {
        assert!(knows("premises"));
        assert!(knows("jobs"));
        assert!(!knows("postcodes"));
    }
}
