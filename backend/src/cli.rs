use clap::{Parser, Subcommand};

use crate::import::{jobs, premises};

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
    Premises {
        /// CSV URL, or path to a local CSV
        #[arg(default_value = premises::DEFAULT_URL)]
        source: String,
    },

    /// Sync jobs
    Jobs {
        /// CSV URL, or path to a local CSV
        #[arg(default_value = jobs::DEFAULT_URL)]
        source: String,
    },

    /// Sync postcode coordinates from postcodes.io
    Postcodes,
}
