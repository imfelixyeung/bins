//! Parsing checks against the sample CSVs in `../data`.
//!
//! These assert the behaviour the rest of the app relies on: NUL-marked missing
//! values, day-first dates, and correct handling of rows that straddle the
//! boundaries of the chunks a streaming HTTP response delivers.

use std::io::Error as IoError;
use std::path::PathBuf;

use backend::import::premises::{PremisesRow, search_postcode};
use backend::import::{RowSpec, csv_reader, jobs::JobRow};
use chrono::Datelike;
use futures_util::stream;
use tokio_util::bytes::Bytes;
use tokio_util::io::StreamReader;

fn sample(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../data")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|err| panic!("reading {}: {err}", path.display()))
}

/// Parses the whole buffer as one chunk, i.e. the easy case.
async fn read_all<R: RowSpec>(bytes: Vec<u8>) -> Vec<R> {
    let chunk = Ok::<Bytes, IoError>(Bytes::from(bytes));
    collect::<R>(csv_reader(StreamReader::new(stream::iter(vec![chunk])))).await
}

/// Parses the same buffer split into fixed-size chunks, mimicking a streamed
/// HTTP body where records routinely straddle chunk boundaries.
async fn read_chunked<R: RowSpec>(bytes: &[u8], chunk_size: usize) -> Vec<R> {
    let chunks: Vec<Result<Bytes, IoError>> = bytes
        .chunks(chunk_size)
        .map(|c| Ok(Bytes::copy_from_slice(c)))
        .collect();
    let reader = csv_reader(StreamReader::new(stream::iter(chunks)));
    collect::<R>(reader).await
}

/// The sample file verbatim, so expectations can be derived from the source
/// instead of committing real address text to this file.
fn read_sample(name: &str) -> String {
    String::from_utf8(sample(name)).expect("utf8 sample")
}

/// Mirrors the importer: NUL bytes are stripped, then a blank field is NULL.
fn source_field(line: &str, index: usize) -> Option<String> {
    let field = line.split(',').nth(index).expect("field index in range");
    if field.is_empty() || field == "\0" {
        None
    } else {
        Some(field.replace('\0', ""))
    }
}

/// Reorders a day-first `DD/MM/YY` source date into the ISO form the importer
/// writes, so date handling is asserted without embedding sample rows.
fn iso_from_day_first(date: &str) -> String {
    let mut parts = date
        .split('/')
        .map(|p| p.parse::<u32>().expect("numeric date part"));
    let day = parts.next().expect("day");
    let month = parts.next().expect("month");
    let two_digit_year = parts.next().expect("year");
    assert_eq!(parts.next(), None, "unexpected extra date field");
    let year = if two_digit_year < 70 {
        2000 + two_digit_year
    } else {
        1900 + two_digit_year
    };
    format!("{year:04}-{month:02}-{day:02}")
}

async fn collect<R: RowSpec>(
    mut reader: csv_async::AsyncReader<impl tokio::io::AsyncRead + Unpin + Send>,
) -> Vec<R> {
    let mut rows = Vec::new();
    let mut record = csv_async::StringRecord::new();
    while reader
        .read_record(&mut record)
        .await
        .expect("reading record")
    {
        rows.push(R::parse(&record).expect("parsing record"));
    }
    rows
}

// ---------------------------------------------------------------- premises

#[tokio::test]
async fn premises_sample_row_count() {
    let rows: Vec<PremisesRow> = read_all(sample("dm_premises.sample.csv")).await;
    assert_eq!(rows.len(), 1000);
}

#[tokio::test]
async fn premises_first_row_matches_source() {
    let raw = read_sample("dm_premises.sample.csv");
    let line = raw.lines().next().expect("sample should not be empty");
    // One column fewer than the COPY payload, which gains the derived key.
    assert_eq!(line.split(',').count(), PremisesRow::COLUMNS.len() - 1);

    let rows: Vec<PremisesRow> = read_all(sample("dm_premises.sample.csv")).await;
    let first = &rows[0];

    // Expectations are derived from the raw line, so no real address has to be
    // committed to this file. The sample's first row exercises the NUL path.
    assert_eq!(source_field(line, 1), None, "first row is NUL-marked");
    assert_eq!(first.id.to_string(), line.split(',').next().unwrap());
    assert_eq!(first.address_room, None);
    assert_eq!(
        first.address_number.as_deref(),
        source_field(line, 2).as_deref()
    );
    assert_eq!(
        first.address_street.as_deref(),
        source_field(line, 3).as_deref()
    );
    assert_eq!(
        first.address_locality.as_deref(),
        source_field(line, 4).as_deref()
    );
    assert_eq!(
        first.address_city.as_deref(),
        source_field(line, 5).as_deref()
    );
    assert_eq!(
        first.address_postcode.as_deref(),
        source_field(line, 6).as_deref()
    );
    assert_eq!(
        first.search_postcode.as_deref(),
        source_field(line, 6)
            .map(|p| search_postcode(&p))
            .as_deref()
    );
}

#[tokio::test]
async fn premises_missing_value_counts_match_source() {
    let rows: Vec<PremisesRow> = read_all(sample("dm_premises.sample.csv")).await;

    let count_none =
        |f: fn(&PremisesRow) -> &Option<String>| rows.iter().filter(|r| f(r).is_none()).count();

    // Verified against a byte-level scan of the sample: 933 NUL address_room,
    // 55 NUL address_number, 6 NUL address_postcode, and no other field is ever
    // blank.
    assert_eq!(count_none(|r| &r.address_room), 933);
    assert_eq!(count_none(|r| &r.address_number), 55);
    assert_eq!(count_none(|r| &r.address_postcode), 6);
    assert_eq!(count_none(|r| &r.address_locality), 0);
    assert_eq!(count_none(|r| &r.address_city), 0);
}

#[tokio::test]
async fn premises_no_value_is_an_empty_string() {
    let rows: Vec<PremisesRow> = read_all(sample("dm_premises.sample.csv")).await;

    for row in &rows {
        for value in [
            &row.address_room,
            &row.address_number,
            &row.address_street,
            &row.address_locality,
            &row.address_city,
            &row.address_postcode,
            &row.search_postcode,
        ]
        .into_iter()
        .flatten()
        {
            assert!(!value.is_empty(), "empty string should be NULL: {row:?}");
            assert!(!value.contains('\0'), "NUL survived: {row:?}");
        }
    }
}

#[tokio::test]
async fn premises_search_postcode_is_derived_from_postcode() {
    let rows: Vec<PremisesRow> = read_all(sample("dm_premises.sample.csv")).await;

    for row in &rows {
        match &row.address_postcode {
            None => assert_eq!(row.search_postcode, None, "{row:?}"),
            Some(postcode) => assert_eq!(
                row.search_postcode.as_deref(),
                Some(search_postcode(postcode).as_str()),
                "{row:?}"
            ),
        }
    }
}

#[tokio::test]
async fn premises_search_postcodes_are_the_source_postcode_without_spaces() {
    let rows: Vec<PremisesRow> = read_all(sample("dm_premises.sample.csv")).await;

    // The expectation is derived straight from the source field rather than by
    // calling the implementation, so this stays a real check. The sample is
    // already upper case, so upper-casing is only covered by the unit tests.
    let mut seen = 0;
    for row in &rows {
        let (Some(postcode), Some(key)) = (&row.address_postcode, &row.search_postcode) else {
            continue;
        };

        assert!(!key.contains(' '), "search key still spaced: {key}");
        assert_eq!(key, &postcode.replace(' ', ""), "wrong search key");
        // Every source postcode in the sample is written spaced, so this loop
        // genuinely exercises the space-stripping on each row.
        assert!(postcode.contains(' '), "expected spaced source postcodes");
        seen += 1;
    }

    assert_eq!(
        seen,
        1000 - 6,
        "every row but the 6 NUL postcodes has a search key"
    );
}

#[tokio::test]
async fn premises_ids_are_unique() {
    let rows: Vec<PremisesRow> = read_all(sample("dm_premises.sample.csv")).await;
    let mut ids: Vec<i32> = rows.iter().map(|r| r.id).collect();
    let total = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), total, "duplicate premises ids in sample");
}

#[tokio::test]
async fn premises_parse_identically_at_every_chunk_size() {
    let bytes = sample("dm_premises.sample.csv");
    let expected: Vec<PremisesRow> = read_all(bytes.clone()).await;
    assert_eq!(expected.len(), 1000);

    // 1 forces every record to span several reads; the others cut at awkward
    // offsets, including mid-field and mid-quote.
    for chunk_size in [1usize, 2, 3, 5, 7, 13, 64, 511, 4096] {
        let actual: Vec<PremisesRow> = read_chunked(&bytes, chunk_size).await;
        assert_eq!(
            actual.len(),
            expected.len(),
            "row count differs at chunk size {chunk_size}"
        );
        assert_eq!(actual, expected, "rows differ at chunk size {chunk_size}");
    }
}

// -------------------------------------------------------------------- jobs

#[tokio::test]
async fn jobs_sample_row_count() {
    let rows: Vec<JobRow> = read_all(sample("dm_jobs.sample.csv")).await;
    assert_eq!(rows.len(), 1000);
}

#[tokio::test]
async fn jobs_first_rows_match_source() {
    let raw = read_sample("dm_jobs.sample.csv");
    let sources: Vec<&str> = raw.lines().collect();
    let rows: Vec<JobRow> = read_all(sample("dm_jobs.sample.csv")).await;
    assert_eq!(rows.len(), sources.len());

    // Checked against the raw day-first source lines so that no real premise id
    // is hard-coded here, and the DD/MM/YY reorder is what gets asserted.
    for (row, source) in rows.iter().zip(&sources).take(3) {
        let fields: Vec<&str> = source.split(',').collect();
        assert_eq!(fields.len(), JobRow::COLUMNS.len());
        assert_eq!(row.premises_id.to_string(), fields[0]);
        assert_eq!(row.bin, fields[1]);
        assert_eq!(row.date.to_string(), iso_from_day_first(fields[2]));
    }
}

#[tokio::test]
async fn jobs_dates_are_read_day_first() {
    let rows: Vec<JobRow> = read_all(sample("dm_jobs.sample.csv")).await;

    for row in &rows {
        let source = row.date.format("%d/%m/%y").to_string();
        assert_eq!(row.date.to_string().len(), 10, "{row:?}");
        assert!(source.ends_with("/26"), "unexpected year in {source}");
    }

    // 24/10/26 only parses if the day comes first.
    let october_24 = rows.iter().find(|r| r.date.day() == 24).expect("a 24th");
    assert_eq!(october_24.date.month(), 10);
}

#[tokio::test]
async fn jobs_bins_come_from_the_expected_set() {
    let rows: Vec<JobRow> = read_all(sample("dm_jobs.sample.csv")).await;

    let mut bins: Vec<&str> = rows.iter().map(|r| r.bin.as_str()).collect();
    bins.sort_unstable();
    bins.dedup();
    assert_eq!(bins, vec!["BLACK", "BROWN", "GREEN"]);
}

#[tokio::test]
async fn jobs_rows_render_iso_dates_for_copy() {
    let rows: Vec<JobRow> = read_all(sample("dm_jobs.sample.csv")).await;

    for row in &rows {
        let fields = row.fields();
        assert_eq!(fields.len(), JobRow::COLUMNS.len());
        assert_eq!(fields[2], row.date.format("%Y-%m-%d").to_string());
        assert_eq!(fields[1], row.bin);
        assert_eq!(fields[0], row.premises_id.to_string());
    }
}

#[tokio::test]
async fn jobs_parse_identically_at_every_chunk_size() {
    let bytes = sample("dm_jobs.sample.csv");
    let expected: Vec<JobRow> = read_all(bytes.clone()).await;
    assert_eq!(expected.len(), 1000);

    for chunk_size in [1usize, 2, 3, 5, 7, 13, 64, 511, 4096] {
        let actual: Vec<JobRow> = read_chunked(&bytes, chunk_size).await;
        assert_eq!(
            actual.len(),
            expected.len(),
            "row count differs at chunk size {chunk_size}"
        );
        assert_eq!(actual, expected, "rows differ at chunk size {chunk_size}");
    }
}

// ------------------------------------------------------- cross-file checks

#[tokio::test]
async fn every_job_premises_id_parses_as_a_premises_id() {
    let jobs: Vec<JobRow> = read_all(sample("dm_jobs.sample.csv")).await;
    let premises: Vec<PremisesRow> = read_all(sample("dm_premises.sample.csv")).await;

    // The samples are independent extracts, so this only asserts the ids are
    // well-formed positive integers rather than that they all resolve.
    assert!(jobs.iter().all(|j| j.premises_id > 0));
    assert!(premises.iter().all(|p| p.id > 0));
}

/// Serialises rows exactly as the importer ships them to `COPY FROM STDIN CSV`.
async fn copy_payload<R: RowSpec>(rows: &[R]) -> String {
    let mut buf: Vec<u8> = Vec::new();
    {
        let mut writer = csv_async::AsyncWriter::from_writer(&mut buf);
        for row in rows {
            writer.write_record(row.fields()).await.unwrap();
        }
        writer.flush().await.unwrap();
    }
    String::from_utf8(buf).expect("utf8 payload")
}

#[tokio::test]
async fn premises_copy_payload_has_every_column() {
    let rows: Vec<PremisesRow> = read_all(sample("dm_premises.sample.csv")).await;
    let payload = copy_payload(&rows).await;
    let lines: Vec<&str> = payload.lines().collect();

    assert_eq!(lines.len(), 1000);
    // No field in the sample needs quoting, which is what makes the naive split
    // below equivalent to a real CSV parse.
    assert!(!payload.contains('"'), "unexpected quoting in payload");
    assert!(
        lines
            .iter()
            .all(|line| line.split(',').count() == PremisesRow::COLUMNS.len()),
        "every line should carry all {} columns",
        PremisesRow::COLUMNS.len()
    );

    // A NUL field becomes an unquoted empty field, which is how
    // `COPY ... FROM STDIN CSV` is told to insert NULL, and the derived search
    // key is appended last. Derived from the raw line, so no real address is
    // committed here.
    let raw = read_sample("dm_premises.sample.csv");
    let line = raw.lines().next().expect("sample should not be empty");
    let search_key = source_field(line, 6)
        .map(|p| search_postcode(&p))
        .unwrap_or_default();
    assert_eq!(
        lines[0],
        format!("{},{}", line.replace('\0', ""), search_key)
    );

    for (line, row) in lines.iter().zip(&rows) {
        let fields: Vec<&str> = line.split(',').collect();
        assert_eq!(fields[0], row.id.to_string());
        assert_eq!(fields[6], row.address_postcode.clone().unwrap_or_default());
        assert_eq!(fields[7], row.search_postcode.clone().unwrap_or_default());
    }
}

#[tokio::test]
async fn jobs_copy_payload_uses_iso_dates() {
    let rows: Vec<JobRow> = read_all(sample("dm_jobs.sample.csv")).await;
    let payload = copy_payload(&rows).await;
    let lines: Vec<&str> = payload.lines().collect();

    assert_eq!(lines.len(), 1000);
    assert!(
        lines
            .iter()
            .all(|line| line.split(',').count() == JobRow::COLUMNS.len())
    );

    // Day-first source becomes ISO on the way in, so the import no longer
    // depends on the session's DateStyle.
    let raw = read_sample("dm_jobs.sample.csv");
    for (line, source) in lines.iter().zip(raw.lines()).take(3) {
        let fields: Vec<&str> = source.split(',').collect();
        assert_eq!(
            line,
            &format!(
                "{},{},{}",
                fields[0],
                fields[1],
                iso_from_day_first(fields[2])
            )
        );
    }

    for (line, row) in lines.iter().zip(&rows) {
        let fields: Vec<&str> = line.split(',').collect();
        assert_eq!(fields[0], row.premises_id.to_string());
        assert_eq!(fields[1], row.bin);
        assert_eq!(fields[2], row.date.format("%Y-%m-%d").to_string());
        assert_eq!(fields[2].len(), 10);
    }
}
