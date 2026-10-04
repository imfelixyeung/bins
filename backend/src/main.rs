use backend::cli::{Cli, Command, SyncTarget};
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=info".into()),
        )
        .init();

    match Cli::parse().command {
        Command::Serve => backend::serve::run().await?,
        Command::Sync { target } => match target {
            SyncTarget::Premises { source } => backend::sync::premises(&source).await?,
            SyncTarget::Jobs { source } => backend::sync::jobs(&source).await?,
            SyncTarget::Postcodes => backend::sync::postcodes().await?,
        },
    }

    Ok(())
}
