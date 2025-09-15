use bytes::Buf;
use chrono::NaiveDate;
use csv::StringRecord;
use futures_util::StreamExt;
use tokio_postgres::types::{ToSql, Type};

pub trait DatabaseSync {
    fn table_name(&self) -> &str;
    fn csv_url(&self) -> &str;
    fn nullable_columns(&self) -> &Vec<String>;
    fn csv_columns(&self) -> &Vec<String>;
    fn csv_column_schema(&self) -> &Vec<Type>;
    fn copy_columns(&self) -> &Vec<String>;

    fn staging_table_name(&self) -> String {
        return format!("staging_{table}", table = self.table_name());
    }

    async fn row_insert_statement(&self, db: &tokio_postgres::Client) -> tokio_postgres::Statement {
        let staging_db_table = self.staging_table_name();
        let columns = self.csv_columns().join(", ");
        let values = (1..=self.csv_columns().len())
            .map(|i| format!("${i}"))
            .collect::<Vec<_>>()
            .join(",");
        let statement = format!("INSERT INTO {staging_db_table} ({columns}) VALUES ({values})");

        let prepared_staging_insert = db
            .prepare_typed(statement.as_str(), &self.csv_column_schema())
            .await
            .unwrap();

        return prepared_staging_insert;
    }

    async fn prepare(&self, db: &tokio_postgres::Client) {
        let db_table = self.table_name();
        let staging_db_table = self.staging_table_name();

        db.batch_execute("SET DateStyle TO 'ISO', 'DMY';")
            .await
            .unwrap();
        db.batch_execute(format!("DROP TABLE IF EXISTS {staging_db_table};",).as_str())
            .await
            .unwrap();
        db.batch_execute(
            format!("CREATE TEMP TABLE {staging_db_table} AS TABLE {db_table} WITH NO DATA;")
                .as_str(),
        )
        .await
        .unwrap();
    }

    fn parse_record_to_db_params(&self, record: &StringRecord) -> Vec<Box<dyn ToSql + Sync>> {
        self.csv_column_schema()
            .iter()
            .enumerate()
            .map(|(i, col_type)| {
                let val = record.get(i).unwrap(); // safe unwrap if CSV is well-formed
                match col_type {
                    &Type::INT8 => {
                        let parsed: i64 = val.parse().unwrap();
                        Box::new(parsed) as Box<dyn ToSql + Sync>
                    }
                    &Type::TEXT => Box::new(val.to_string()) as Box<dyn ToSql + Sync>,
                    &Type::DATE => {
                        // Format: "mm/dd/yy"
                        println!("{val}");
                        let parsed = NaiveDate::parse_from_str(val, "%d/%m/%y").unwrap();
                        Box::new(parsed) as Box<dyn ToSql + Sync>
                    }
                    _ => unreachable!(),
                }
            })
            .collect()
    }

    async fn fetch(&self, db: &tokio_postgres::Client) {
        let prepared_staging_insert = self.row_insert_statement(&db).await;

        let request_url = self.csv_url();
        println!("Fetching {}", request_url);

        let client = reqwest::Client::new();
        let response = client
            .get(request_url)
            .send()
            .await
            .expect("Unable to fetch csv file");

        println!("Fetch finished");

        // get the byte stream
        let mut stream = response.bytes_stream();

        let mut counter = 1;

        // hold content that were not processed from previous loop
        let mut previous_content: Vec<u8> = vec![];

        while let Some(item) = stream.next().await {
            // get the current chunk and convert into vector of u8
            let item_content = item.expect("unknown item").to_vec();
            let mut content: Vec<_> = item_content
                .iter()
                .filter(|x| x != &&0x0)
                .cloned()
                .collect();

            // insert the previous contents into start of this chunk for further processing
            if previous_content.len() > 0 {
                content.splice(0..0, previous_content);
            }

            println!("Chunk {} size: {}", counter, content.len());
            counter += 1;

            let lines: Vec<_> = content
                .split(|b| b == &0xA)
                .map(|line| line.strip_suffix(&[0xD]).unwrap_or(line))
                .map(|slice| slice.to_vec())
                .collect();

            let last_line = lines.last().expect("last line not found");
            // previous_content = &last_line.to_vec();
            previous_content = last_line.clone();

            let current_content: Vec<_> = Vec::from(&lines[..lines.len() - 1])
                .iter()
                .flat_map(|slice| {
                    let mut new = slice.to_vec();
                    new.push(0xA);
                    return new;
                })
                .collect();

            let mut reader = csv::Reader::from_reader(current_content.reader());

            for result in reader.records() {
                let record = result.expect("error reading record");

                let params = self.parse_record_to_db_params(&record);

                // Convert Vec<Box<...>> → Vec<&dyn ToSql>
                let param_refs: Vec<&(dyn ToSql + Sync)> = params.iter().map(|p| &**p).collect();

                db.query(&prepared_staging_insert, &param_refs)
                    .await
                    .unwrap();
            }
        }
    }

    async fn postprocess(&self, db: &tokio_postgres::Client) {
        self.postprocess_custom(&db).await;

        let staging_db_table = self.staging_table_name();
        for column in self.nullable_columns().iter() {
            println!("Replacing nulls for {column}");
            db.batch_execute(
                format!("UPDATE {staging_db_table} SET {column} = NULL WHERE {column} = '';")
                    .as_str(),
            )
            .await
            .unwrap();
        }
    }
    async fn postprocess_custom(&self, db: &tokio_postgres::Client);

    async fn commit(&self, db: &tokio_postgres::Client) {
        let db_table = self.table_name();
        let staging_db_table = self.staging_table_name();

        let columns = self.copy_columns().join(", ");

        println!("Replacing production table");
        db.batch_execute(
            format!(
                "
        BEGIN TRANSACTION;
        DO $$
        BEGIN
        IF (SELECT COUNT(*) FROM {staging_db_table}) > 0 THEN

            -- delete all rows from table
            DELETE FROM {db_table};

            -- copy data from staging to table
            INSERT INTO {db_table}
            ({columns})
            SELECT
            {columns}
            FROM {staging_db_table};

        END IF;
        END $$;
        COMMIT TRANSACTION;
        
        "
            )
            .as_str(),
        )
        .await
        .unwrap();
    }

    async fn process(&self, db: &tokio_postgres::Client) {
        self.prepare(&db).await;
        self.fetch(&db).await;
        self.postprocess(&db).await;
        self.commit(&db).await;
    }
}

pub struct DatabaseSyncPremises {
    pub table_name: String,
    pub csv_url: String,
    pub nullable_columns: Vec<String>,
    pub csv_columns: Vec<String>,
    pub csv_column_schema: Vec<Type>,
    pub copy_columns: Vec<String>,
}

impl DatabaseSync for DatabaseSyncPremises {
    fn table_name(&self) -> &str {
        &self.table_name
    }
    fn csv_url(&self) -> &str {
        &self.csv_url
    }
    fn nullable_columns(&self) -> &Vec<String> {
        &self.nullable_columns
    }
    fn csv_columns(&self) -> &Vec<String> {
        &self.csv_columns
    }
    fn csv_column_schema(&self) -> &Vec<Type> {
        &self.csv_column_schema
    }
    fn copy_columns(&self) -> &Vec<String> {
        &self.copy_columns
    }

    async fn postprocess_custom(&self, db: &tokio_postgres::Client) {
        let staging_db_table = self.staging_table_name();
        println!("Setting search postcode");
        db.batch_execute(
        format!("UPDATE {staging_db_table} SET search_postcode = upper(replace(address_postcode, ' ', ''));")
            .as_str(),
            )
            .await
            .unwrap();
    }
}

pub struct DatabaseSyncJobs {
    pub table_name: String,
    pub csv_url: String,
    pub nullable_columns: Vec<String>,
    pub csv_columns: Vec<String>,
    pub csv_column_schema: Vec<Type>,
    pub copy_columns: Vec<String>,
}

impl DatabaseSync for DatabaseSyncJobs {
    fn table_name(&self) -> &str {
        &self.table_name
    }
    fn csv_url(&self) -> &str {
        &self.csv_url
    }
    fn nullable_columns(&self) -> &Vec<String> {
        &self.nullable_columns
    }
    fn csv_columns(&self) -> &Vec<String> {
        &self.csv_columns
    }
    fn csv_column_schema(&self) -> &Vec<Type> {
        &self.csv_column_schema
    }
    fn copy_columns(&self) -> &Vec<String> {
        &self.copy_columns
    }

    async fn postprocess_custom(&self, _db: &tokio_postgres::Client) {}
}
