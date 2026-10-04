use anyhow::{Context, Result};
use tracing::info;

use crate::db;
use crate::routes;

/// Binds all interfaces by default so the API is reachable from outside the
/// container.
const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: &str = "3000";

pub async fn run() -> Result<()> {
    let url = db::database_url();
    let pool = db::connect(&url).await?;
    // Migrations are owned by the app rather than a separate compose service.
    db::migrate(&pool).await?;

    let host = std::env::var("HOST").unwrap_or_else(|_| DEFAULT_HOST.to_owned());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| DEFAULT_PORT.to_owned())
        .parse()
        .context("PORT must be a valid TCP port number")?;
    let listener = tokio::net::TcpListener::bind((host.as_str(), port))
        .await
        .with_context(|| format!("binding {host}:{port}"))?;

    info!(address = %listener.local_addr()?, "listening");
    // The pool is handed to the router, which closes it on shutdown.
    axum::serve(listener, routes::router(pool))
        .await
        .context("serving requests")
}
