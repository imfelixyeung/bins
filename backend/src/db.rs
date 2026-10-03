use anyhow::{Context, Result};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub const DEFAULT_DATABASE_URL: &str = "postgres://postgres:postgres@localhost:5432/db";

pub fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_owned())
}

pub async fn connect(url: &str) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
        .with_context(|| format!("connecting to {url}"))
}

pub async fn migrate(pool: &PgPool) -> Result<()> {
    sqlx::migrate!()
        .run(pool)
        .await
        .context("running migrations")
}
