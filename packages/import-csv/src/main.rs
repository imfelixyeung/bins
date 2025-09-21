use crate::sync::DatabaseSync;
use std::env;
use tokio_postgres::types::Type;

mod sync;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Read arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        log::error!("Usage: {} <premises|jobs>", args[0]);
        return;
    }

    let task = match args[1].as_str() {
        "premises" | "jobs" => args[1].as_str(),
        _ => {
            log::error!("Unknown argument '{}'. Use 'premises' or 'jobs'", args[1]);
            return;
        }
    };

    // Read database connection info from environment variables
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "localhost".to_string());

    // connect to database
    let (db, connection) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
        .await
        .unwrap();

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    match task {
        "premises" => {
            let premises = sync::DatabaseSyncPremises {
                csv_url: "https://opendata.leeds.gov.uk/downloads/bins/dm_premises.csv".to_string(),
                table_name: "dm_premises".to_string(),
                nullable_columns: vec![
                    "address_room".to_string(),
                    "address_number".to_string(),
                    "address_street".to_string(),
                    "address_locality".to_string(),
                    "address_city".to_string(),
                    "address_postcode".to_string(),
                ],
                csv_columns: vec![
                    "id".to_string(),
                    "address_room".to_string(),
                    "address_number".to_string(),
                    "address_street".to_string(),
                    "address_locality".to_string(),
                    "address_city".to_string(),
                    "address_postcode".to_string(),
                ],
                csv_column_schema: vec![
                    Type::INT8,
                    Type::TEXT,
                    Type::TEXT,
                    Type::TEXT,
                    Type::TEXT,
                    Type::TEXT,
                    Type::TEXT,
                ],
                copy_columns: vec![
                    "id".to_string(),
                    "address_room".to_string(),
                    "address_number".to_string(),
                    "address_street".to_string(),
                    "address_locality".to_string(),
                    "address_city".to_string(),
                    "address_postcode".to_string(),
                    "search_postcode".to_string(),
                ],
            };

            premises.process(&db).await;
        }

        "jobs" => {
            let jobs = sync::DatabaseSyncJobs {
                csv_url: "https://opendata.leeds.gov.uk/downloads/bins/dm_jobs.csv".to_string(),
                table_name: "dm_jobs".to_string(),
                nullable_columns: vec![],
                csv_columns: vec![
                    "premises_id".to_string(),
                    "bin".to_string(),
                    "date".to_string(),
                ],
                csv_column_schema: vec![Type::INT8, Type::TEXT, Type::DATE],
                copy_columns: vec![
                    "premises_id".to_string(),
                    "bin".to_string(),
                    "date".to_string(),
                ],
            };

            jobs.process(&db).await;
        }

        _ => {
            log::error!("Unknown argument '{}'. Use 'premises' or 'jobs'", args[1]);
        }
    }

    log::info!("Done!");
}
