use anyhow::Result;
use tracing::info;

use crate::db;
use crate::import::{self, jobs::JobRow, premises::PremisesRow};
use crate::postcodes;
use crate::source::Source;

pub async fn premises(raw_source: &str) -> Result<()> {
    let source = Source::parse(raw_source);
    info!(source = %source.describe(), "syncing");

    let pool = db::connect(&db::database_url()).await?;
    db::migrate(&pool).await?;

    let rows = import::run::<PremisesRow>(&pool, &source).await?;

    pool.close().await;
    info!(rows, "premises sync complete");
    Ok(())
}

pub async fn jobs(raw_source: &str) -> Result<()> {
    let source = Source::parse(raw_source);
    info!(source = %source.describe(), "syncing");

    let pool = db::connect(&db::database_url()).await?;
    db::migrate(&pool).await?;

    let rows = import::run::<JobRow>(&pool, &source).await?;

    pool.close().await;
    info!(rows, "jobs sync complete");
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
