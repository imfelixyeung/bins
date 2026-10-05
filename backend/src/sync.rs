//! Imports from the upstream CSVs, skipped when the file behind them has not
//! changed, with how far each run got recorded for `GET /api/status`.
//!
//! The skip is the one `apps/worker/etag.ts` put in front of its import scripts.
//! The file is asked about first and only downloaded when the etag differs from
//! the one stored, which is what leaves the daily cron cheap on the days the
//! council publishes nothing. The etag is only written once the import has
//! succeeded, so a run that fails is tried again rather than written off.

use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

use crate::db;
use crate::etag;
use crate::import::{self, RowSpec, jobs::JobRow, premises::PremisesRow};
use crate::postcodes;
use crate::source::Source;
use crate::sync_run::{self, Outcome};

pub async fn premises(raw_source: &str) -> Result<()> {
    let source = Source::parse(raw_source);
    info!(source = %source.describe(), "syncing");

    let pool = db::connect(&db::database_url()).await?;
    db::migrate(&pool).await?;

    dataset::<PremisesRow>(&pool, sync_run::PREMISES, &source).await?;

    pool.close().await;
    Ok(())
}

pub async fn jobs(raw_source: &str) -> Result<()> {
    let source = Source::parse(raw_source);
    info!(source = %source.describe(), "syncing");

    let pool = db::connect(&db::database_url()).await?;
    db::migrate(&pool).await?;

    dataset::<JobRow>(&pool, sync_run::JOBS, &source).await?;

    pool.close().await;
    Ok(())
}

/// Fills in coordinates for postcodes that do not have any yet.
///
/// Unlike the other syncs this one reads no file: it asks postcodes.io for a
/// postcode the database is missing and stores what comes back, so it is worth
/// running often rather than once a day.
pub async fn postcodes() -> Result<()> {
    let pool = db::connect(&db::database_url()).await?;
    db::migrate(&pool).await?;

    match postcodes::sync(&pool).await? {
        Some(postcode) => info!(%postcode, "postcodes sync complete"),
        None => info!("no postcodes to fetch"),
    }

    pool.close().await;
    Ok(())
}

/// Imports one dataset, recording the run as it goes.
///
/// The run is recorded whether the import worked or not, which is the point of
/// it: a sync that stops half way through leaves a record saying so, rather than
/// the status page carrying on as though nothing were happening.
async fn dataset<S: RowSpec>(pool: &PgPool, target: &str, source: &Source) -> Result<()> {
    let run = sync_run::start(pool, target, &source.describe()).await?;

    let (outcome, error) = match gated::<S>(pool, source).await {
        Ok(Imported::Synced { rows }) => (Outcome::Synced { rows }, None),
        Ok(Imported::Unchanged) => (Outcome::Unchanged, None),
        Err(error) => (Outcome::Failed(format!("{error:#}")), Some(error)),
    };

    sync_run::finish(pool, &run, outcome).await?;

    match error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

/// What a gated import did.
enum Imported {
    /// The tables now hold what upstream publishes, with this many rows.
    Synced { rows: i64 },
    /// Upstream has not changed since the last sync, so nothing was imported.
    Unchanged,
}

/// Imports `source` into its table, unless upstream has not changed.
///
/// A file on disk is always imported: there is no upstream behind it to have
/// changed, and the etag table is keyed by URL.
async fn gated<S: RowSpec>(pool: &PgPool, source: &Source) -> Result<Imported> {
    let Some(url) = source.url() else {
        return Ok(Imported::Synced {
            rows: rows(import::run::<S>(pool, source).await?),
        });
    };

    let check = etag::check(pool, url).await?;

    if etag::unchanged(check.stored.as_ref(), &check.latest) {
        // The data is as it was, but the check still happened, so it is recorded
        // and `lastChecked` moves on.
        etag::checked(pool, url, &check.latest).await?;
        info!(%url, "upstream has not changed, nothing to import");
        return Ok(Imported::Unchanged);
    }

    let imported = import::run::<S>(pool, source).await?;

    // Written after the import rather than before it, so a run that failed part
    // way through is run again on the next tick.
    etag::store(pool, url, &check.latest).await?;

    Ok(Imported::Synced {
        rows: rows(imported),
    })
}

/// A row count as the `sync_runs` column holds it. A CSV long enough for this to
/// matter is not one this project has to hold, but it is not worth failing over.
fn rows(imported: u64) -> i64 {
    i64::try_from(imported).unwrap_or(i64::MAX)
}
