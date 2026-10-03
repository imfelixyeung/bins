mod cli;
mod routes;
mod serve;
mod sync;

use clap::Parser;

use crate::cli::{Cli, Command, SyncTarget};

#[tokio::main]
async fn main() {
    match Cli::parse().command {
        Command::Serve => serve::run().await,
        Command::Sync { target } => match target {
            SyncTarget::Premises => sync::premises().await,
            SyncTarget::Jobs => sync::jobs().await,
        },
    }
}
