use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "backend",
    bin_name = "backend",
    about = "Household waste collection API"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the HTTP API server
    Serve,

    /// Sync data from the upstream provider
    Sync {
        #[command(subcommand)]
        target: SyncTarget,
    },
}

#[derive(Subcommand)]
pub enum SyncTarget {
    /// Sync premises
    Premises,

    /// Sync jobs
    Jobs,
}
