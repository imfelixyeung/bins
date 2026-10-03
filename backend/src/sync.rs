use anyhow::Result;
use tracing::info;

use crate::db;
use crate::import::{self, jobs::JobRow, premises::PremisesRow};
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
