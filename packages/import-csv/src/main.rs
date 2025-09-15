use crate::sync::DatabaseSync;
// use
mod sync;

#[tokio::main]
async fn main() {
    // connect to database
    let (db, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=db port=56432 connect_timeout=5",
        tokio_postgres::NoTls,
    )
    .await
    .unwrap();

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

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
    };

    premises.prepare(&db).await;
    premises.fetch(&db).await;
    premises.postprocess(&db).await;
    premises.commit(&db).await;

    println!("Done!");
}
