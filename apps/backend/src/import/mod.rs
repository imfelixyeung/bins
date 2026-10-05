pub mod jobs;
pub mod premises;

use anyhow::{Context, Result};
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgCopyIn, PgPoolCopyExt, Postgres};
use sqlx::{AssertSqlSafe, PgPool};
use tracing::{info, warn};

use crate::source::Source;

/// Rows buffered before each `COPY` round trip. Bounds peak memory regardless of
/// how large the source CSV is.
const BATCH_ROWS: usize = 5_000;

/// One CSV record mapped onto the columns of a destination table.
pub trait RowSpec: Sized {
    const TABLE: &'static str;
    const COLUMNS: &'static [&'static str];

    fn parse(record: &csv_async::StringRecord) -> Result<Self>;

    /// Values aligned with [`RowSpec::COLUMNS`]. An empty value is written as an
    /// unquoted field, which `COPY ... FROM STDIN CSV` reads back as SQL NULL.
    fn fields(&self) -> Vec<String>;
}

/// Renders an optional value for `COPY`, where empty means NULL.
pub fn nullable(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}

/// Upstream marks missing values with a NUL byte rather than leaving the field
/// empty, so strip them before deciding whether a value is absent.
pub fn strip_nuls(raw: &str) -> String {
    raw.chars().filter(|c| *c != '\0').collect()
}

pub fn null_if_empty(raw: &str) -> Option<String> {
    let cleaned = strip_nuls(raw);
    (!cleaned.is_empty()).then_some(cleaned)
}

pub fn parse_id(raw: &str) -> Result<i32> {
    strip_nuls(raw)
        .trim()
        .parse::<i32>()
        .with_context(|| format!("parsing id from {raw:?}"))
}

fn staging_table(table: &str) -> String {
    format!("staging_{table}")
}

/// The CSV reader used for every import. Upstream files have no header row.
///
/// Shared with the integration tests so they exercise the same parser
/// configuration as production.
pub fn csv_reader<R>(bytes: R) -> csv_async::AsyncReader<R>
where
    R: tokio::io::AsyncRead + Unpin + Send,
{
    let mut builder = csv_async::AsyncReaderBuilder::new();
    builder.has_headers(false);
    builder.create_reader(bytes)
}

/// Streams `source` into the destination table.
///
/// Rows land in a staging table first and only replace the live table once the
/// whole file has been read, so a truncated download or a parse error leaves the
/// existing data untouched.
pub async fn run<S: RowSpec>(pool: &PgPool, source: &Source) -> Result<u64> {
    let table = S::TABLE;
    let staging = staging_table(table);

    prepare(pool, table, &staging).await?;

    let staged = match copy_rows::<S>(pool, &staging, source).await {
        Ok(rows) => rows,
        Err(err) => {
            let _ = sqlx::query(AssertSqlSafe(format!("DROP TABLE IF EXISTS {staging}")))
                .execute(pool)
                .await;
            return Err(err);
        }
    };

    let live = swap::<S>(pool, table, &staging).await?;
    reset_sequence::<S>(pool).await?;

    sqlx::query(AssertSqlSafe(format!("DROP TABLE IF EXISTS {staging}")))
        .execute(pool)
        .await
        .with_context(|| format!("dropping {staging}"))?;

    if live == staged {
        info!(table, rows = live, "import complete");
    } else {
        warn!(
            table,
            staged, live, "import skipped, existing data left in place"
        );
    }
    Ok(live)
}

/// `CREATE TABLE AS ... WITH NO DATA` copies the column types without the
/// primary key, indexes or constraints, which keeps the `COPY` fast.
async fn prepare(pool: &PgPool, table: &str, staging: &str) -> Result<()> {
    sqlx::query(AssertSqlSafe(format!("DROP TABLE IF EXISTS {staging}")))
        .execute(pool)
        .await
        .with_context(|| format!("dropping stale {staging}"))?;
    sqlx::query(AssertSqlSafe(format!(
        "CREATE TABLE {staging} AS TABLE {table} WITH NO DATA"
    )))
    .execute(pool)
    .await
    .with_context(|| format!("creating {staging}"))?;
    Ok(())
}

async fn copy_rows<S: RowSpec>(pool: &PgPool, staging: &str, source: &Source) -> Result<u64> {
    let mut reader = csv_reader(source.open().await?);

    let mut copy = pool
        .copy_in_raw(&format!(
            "COPY {staging} ({columns}) FROM STDIN CSV",
            columns = S::COLUMNS.join(", ")
        ))
        .await
        .with_context(|| format!("starting COPY into {staging}"))?;

    let ncols = S::COLUMNS.len();
    let mut batch: Vec<String> = Vec::with_capacity(BATCH_ROWS * ncols);
    let mut record = csv_async::StringRecord::new();
    let mut rows = 0u64;
    let mut pending = 0usize;

    let outcome: Result<()> = async {
        while reader.read_record(&mut record).await? {
            let row = S::parse(&record)?;
            batch.extend(row.fields());
            rows += 1;
            pending += 1;
            if pending == BATCH_ROWS {
                flush(&mut copy, &mut batch, ncols).await?;
                pending = 0;
            }
        }
        flush(&mut copy, &mut batch, ncols).await?;
        Ok(())
    }
    .await;

    if let Err(err) = outcome {
        let _ = copy.abort("import aborted").await;
        return Err(err.context("reading csv rows"));
    }

    let copied = copy.finish().await.context("finishing COPY")?;
    info!(staging, rows = copied, "staged rows");
    Ok(copied)
}

/// Serialises buffered rows into `COPY` payload bytes and ships them off. The
/// byte buffer is dropped here so it never grows with the size of the source.
async fn flush(
    copy: &mut PgCopyIn<PoolConnection<Postgres>>,
    batch: &mut Vec<String>,
    ncols: usize,
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let mut buf: Vec<u8> = Vec::with_capacity(batch.len() * 24);
    {
        let mut writer = csv_async::AsyncWriter::from_writer(&mut buf);
        for row in batch.chunks(ncols) {
            writer.write_record(row).await?;
        }
        writer.flush().await?;
    }

    copy.send(buf.as_slice()).await?;
    batch.clear();
    Ok(())
}

/// Replaces the live table with the staged rows, atomically. An empty staging
/// table means the download produced nothing, so the live table is left alone.
/// Returns the number of rows the live table now holds.
async fn swap<S: RowSpec>(pool: &PgPool, table: &str, staging: &str) -> Result<u64> {
    let mut tx = pool.begin().await?;

    let staged: i64 =
        sqlx::query_scalar::<_, i64>(AssertSqlSafe(format!("SELECT count(*) FROM {staging}")))
            .fetch_one(&mut *tx)
            .await
            .with_context(|| format!("counting {staging}"))?;

    if staged == 0 {
        warn!(table, "no rows staged, keeping existing data");
    } else {
        let columns = S::COLUMNS.join(", ");
        sqlx::query(AssertSqlSafe(format!("DELETE FROM {table}")))
            .execute(&mut *tx)
            .await
            .with_context(|| format!("clearing {table}"))?;
        sqlx::query(AssertSqlSafe(format!(
            "INSERT INTO {table} ({columns}) SELECT {columns} FROM {staging}"
        )))
        .execute(&mut *tx)
        .await
        .with_context(|| format!("populating {table}"))?;
    }

    let live: i64 =
        sqlx::query_scalar::<_, i64>(AssertSqlSafe(format!("SELECT count(*) FROM {table}")))
            .fetch_one(&mut *tx)
            .await
            .with_context(|| format!("counting {table}"))?;

    tx.commit().await?;
    Ok(live as u64)
}

/// The CSV carries explicit ids, so the `serial` sequence never advances during
/// the import and has to be re-synchronised or the next insert collides.
async fn reset_sequence<S: RowSpec>(pool: &PgPool) -> Result<()> {
    let table = S::TABLE;
    sqlx::query(AssertSqlSafe(format!(
        "SELECT setval(
             pg_get_serial_sequence('{table}', 'id'),
             COALESCE((SELECT max(id) FROM {table}), 1),
             EXISTS (SELECT 1 FROM {table})
         )"
    )))
    .execute(pool)
    .await
    .with_context(|| format!("resetting {table} id sequence"))?;
    Ok(())
}
