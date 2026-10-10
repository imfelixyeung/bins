use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use csv_async::StringRecord;

use super::{RowSpec, parse_id, strip_nuls};

pub const DEFAULT_URL: &str = "https://opendata.leeds.gov.uk/downloads/bins/dm_jobs.csv";

/// The table the jobs sync fills, as the `RowSpec` names it.
pub const fn table() -> &'static str {
    <JobRow as RowSpec>::TABLE
}

/// Fields present in the source CSV, in order.
const CSV_FIELDS: usize = 3;

/// Upstream writes dates as `DD/MM/YY`, e.g. `02/11/26` is 2 November 2026.
const SOURCE_DATE_FORMAT: &str = "%d/%m/%y";

/// Dates are emitted in ISO form so Postgres parses them the same way
/// regardless of the session's `DateStyle`.
const TARGET_DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobRow {
    pub premises_id: i32,
    pub bin: String,
    pub date: NaiveDate,
}

impl RowSpec for JobRow {
    const TABLE: &'static str = "dm_jobs";
    const COLUMNS: &'static [&'static str] = &["premises_id", "bin", "date"];

    fn parse(record: &StringRecord) -> Result<Self> {
        if record.len() != CSV_FIELDS {
            bail!(
                "expected {CSV_FIELDS} fields in jobs row, found {}",
                record.len()
            );
        }

        let field = |index: usize| record.get(index).unwrap_or_default();

        let bin = strip_nuls(field(1));
        if bin.is_empty() {
            bail!("jobs row is missing a bin");
        }

        let raw_date = strip_nuls(field(2));
        let date = NaiveDate::parse_from_str(&raw_date, SOURCE_DATE_FORMAT)
            .with_context(|| format!("parsing job date {raw_date:?}, expected DD/MM/YY"))?;

        Ok(Self {
            premises_id: parse_id(field(0))?,
            bin,
            date,
        })
    }

    fn fields(&self) -> Vec<String> {
        vec![
            self.premises_id.to_string(),
            self.bin.clone(),
            self.date.format(TARGET_DATE_FORMAT).to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use super::*;

    fn record(fields: &[&str]) -> StringRecord {
        StringRecord::from(fields.iter().map(|f| f.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn parses_day_first_dates() {
        let row = JobRow::parse(&record(&["4242", "GREEN", "02/11/26"])).unwrap();

        assert_eq!(row.premises_id, 4_242);
        assert_eq!(row.bin, "GREEN");
        assert_eq!(row.date.year(), 2026);
        assert_eq!(row.date.month(), 11);
        assert_eq!(row.date.day(), 2);
    }

    #[test]
    fn day_and_month_are_not_swapped() {
        // 24 cannot be a month, so this only parses as 24 October.
        let row = JobRow::parse(&record(&["1", "BLACK", "24/10/26"])).unwrap();
        assert_eq!((row.date.day(), row.date.month()), (24, 10));

        // 03/10/26 is 3 October, not 10 March.
        let row = JobRow::parse(&record(&["1", "BLACK", "03/10/26"])).unwrap();
        assert_eq!((row.date.day(), row.date.month()), (3, 10));
    }

    #[test]
    fn accepts_single_digit_days() {
        let row = JobRow::parse(&record(&["1", "BLACK", "2/11/26"])).unwrap();
        assert_eq!((row.date.day(), row.date.month()), (2, 11));
    }

    #[test]
    fn maps_two_digit_years() {
        let row = JobRow::parse(&record(&["1", "BLACK", "01/01/99"])).unwrap();
        assert_eq!(row.date.year(), 1999);

        let row = JobRow::parse(&record(&["1", "BLACK", "01/01/68"])).unwrap();
        assert_eq!(row.date.year(), 2068);
    }

    #[test]
    fn parses_leap_day() {
        let row = JobRow::parse(&record(&["1", "BLACK", "29/02/24"])).unwrap();
        assert_eq!(
            (row.date.year(), row.date.month(), row.date.day()),
            (2024, 2, 29)
        );
    }

    #[test]
    fn rejects_an_impossible_date() {
        assert!(JobRow::parse(&record(&["1", "BLACK", "31/02/26"])).is_err());
        assert!(JobRow::parse(&record(&["1", "BLACK", "29/02/26"])).is_err());
    }

    #[test]
    fn rejects_us_style_dates_that_look_plausible() {
        // `01/13/26` is 13 January if read month-first, but month 13 does not
        // exist, so a day-first reader must reject it.
        let err = JobRow::parse(&record(&["1", "BLACK", "01/13/26"])).unwrap_err();
        assert!(format!("{err:#}").contains("DD/MM/YY"), "{err:#}");
    }

    #[test]
    fn rejects_a_non_date_value() {
        let err = JobRow::parse(&record(&["1", "BLACK", "not-a-date"])).unwrap_err();
        assert!(format!("{err:#}").contains("DD/MM/YY"), "{err:#}");
    }

    #[test]
    fn rejects_wrong_field_count() {
        let err = JobRow::parse(&record(&["1", "BLACK"])).unwrap_err();
        assert!(err.to_string().contains("expected 3 fields"), "{err}");
    }

    #[test]
    fn rejects_a_missing_bin() {
        assert!(JobRow::parse(&record(&["1", "", "02/11/26"])).is_err());
        assert!(JobRow::parse(&record(&["1", "\0", "02/11/26"])).is_err());
    }

    #[test]
    fn rejects_a_non_numeric_premises_id() {
        assert!(JobRow::parse(&record(&["abc", "BLACK", "02/11/26"])).is_err());
    }

    #[test]
    fn renders_dates_in_iso_for_copy() {
        let row = JobRow::parse(&record(&["4242", "GREEN", "02/11/26"])).unwrap();
        assert_eq!(row.fields(), vec!["4242", "GREEN", "2026-11-02"]);
    }

    #[test]
    fn fields_line_up_with_declared_columns() {
        let row = JobRow::parse(&record(&["1", "BLACK", "02/11/26"])).unwrap();
        assert_eq!(row.fields().len(), JobRow::COLUMNS.len());
    }
}
