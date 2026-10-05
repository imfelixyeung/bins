use anyhow::{Result, bail};
use csv_async::StringRecord;

use super::{RowSpec, null_if_empty, nullable, parse_id, strip_nuls};

pub const DEFAULT_URL: &str = "https://opendata.leeds.gov.uk/downloads/bins/dm_premises.csv";

/// Fields present in the source CSV, in order.
const CSV_FIELDS: usize = 7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PremisesRow {
    pub id: i32,
    pub address_room: Option<String>,
    pub address_number: Option<String>,
    pub address_street: Option<String>,
    pub address_locality: Option<String>,
    pub address_city: Option<String>,
    pub address_postcode: Option<String>,
    pub search_postcode: Option<String>,
}

impl RowSpec for PremisesRow {
    const TABLE: &'static str = "dm_premises";
    const COLUMNS: &'static [&'static str] = &[
        "id",
        "address_room",
        "address_number",
        "address_street",
        "address_locality",
        "address_city",
        "address_postcode",
        "search_postcode",
    ];

    fn parse(record: &StringRecord) -> Result<Self> {
        if record.len() != CSV_FIELDS {
            bail!(
                "expected {CSV_FIELDS} fields in premises row, found {}",
                record.len()
            );
        }

        let field = |index: usize| record.get(index).unwrap_or_default();

        let address_postcode = null_if_empty(field(6));

        Ok(Self {
            id: parse_id(field(0))?,
            address_room: null_if_empty(field(1)),
            address_number: null_if_empty(field(2)),
            address_street: null_if_empty(field(3)),
            address_locality: null_if_empty(field(4)),
            address_city: null_if_empty(field(5)),
            search_postcode: address_postcode.as_deref().map(search_postcode),
            address_postcode,
        })
    }

    fn fields(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            nullable(&self.address_room),
            nullable(&self.address_number),
            nullable(&self.address_street),
            nullable(&self.address_locality),
            nullable(&self.address_city),
            nullable(&self.address_postcode),
            nullable(&self.search_postcode),
        ]
    }
}

/// The legacy importer derived this in SQL as
/// `upper(replace(address_postcode, ' ', ''))`.
pub fn search_postcode(postcode: &str) -> String {
    strip_nuls(postcode).replace(' ', "").to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(fields: &[&str]) -> StringRecord {
        StringRecord::from(fields.iter().map(|f| f.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn parses_a_fully_populated_row() {
        let row = PremisesRow::parse(&record(&[
            "1001",
            "1",
            "2",
            "FAKE STREET",
            "TEST LOCALITY",
            "EXAMPLETON",
            "ZZ1 1ZZ",
        ]))
        .unwrap();

        assert_eq!(row.id, 1_001);
        assert_eq!(row.address_room.as_deref(), Some("1"));
        assert_eq!(row.address_number.as_deref(), Some("2"));
        assert_eq!(row.address_street.as_deref(), Some("FAKE STREET"));
        assert_eq!(row.address_locality.as_deref(), Some("TEST LOCALITY"));
        assert_eq!(row.address_city.as_deref(), Some("EXAMPLETON"));
        assert_eq!(row.address_postcode.as_deref(), Some("ZZ1 1ZZ"));
        assert_eq!(row.search_postcode.as_deref(), Some("ZZ11ZZ"));
    }

    #[test]
    fn nul_byte_marks_a_missing_value() {
        let row = PremisesRow::parse(&record(&[
            "1001",
            "\0",
            "\0",
            "FAKE STREET",
            "TEST LOCALITY",
            "EXAMPLETON",
            "ZZ1 1ZZ",
        ]))
        .unwrap();

        assert_eq!(row.address_room, None);
        assert_eq!(row.address_number, None);
        assert_eq!(row.address_street.as_deref(), Some("FAKE STREET"));
    }

    #[test]
    fn nul_postcode_leaves_search_postcode_absent() {
        let row = PremisesRow::parse(&record(&["1", "", "", "", "", "", "\0"])).unwrap();

        assert_eq!(row.address_postcode, None);
        assert_eq!(row.search_postcode, None);
    }

    #[test]
    fn empty_fields_are_treated_as_missing() {
        let row = PremisesRow::parse(&record(&["1", "", "", "", "", "", ""])).unwrap();

        assert_eq!(row.address_postcode, None);
        assert_eq!(row.search_postcode, None);
    }

    #[test]
    fn nul_embedded_in_a_value_is_stripped() {
        // Matches the legacy importer, which filtered NUL bytes out of the whole
        // byte stream before splitting on commas.
        let row = PremisesRow::parse(&record(&[
            "1",
            "\0",
            "",
            "FA\0KE STREET",
            "",
            "",
            "ZZ1\0 1ZZ",
        ]))
        .unwrap();

        assert_eq!(row.address_street.as_deref(), Some("FAKE STREET"));
        assert_eq!(row.address_postcode.as_deref(), Some("ZZ1 1ZZ"));
        assert_eq!(row.search_postcode.as_deref(), Some("ZZ11ZZ"));
    }

    #[test]
    fn postcode_search_key_is_uppercased_and_unspaced() {
        assert_eq!(search_postcode("zz1 1zz"), "ZZ11ZZ");
        assert_eq!(search_postcode("ZX9 9ZX"), "ZX99ZX");
        assert_eq!(search_postcode("ZZ11ZZ"), "ZZ11ZZ");
    }

    #[test]
    fn missing_fields_render_as_empty_for_copy() {
        let row = PremisesRow::parse(&record(&[
            "1001",
            "\0",
            "1",
            "FAKE STREET",
            "TEST LOCALITY",
            "EXAMPLETON",
            "ZZ1 1ZZ",
        ]))
        .unwrap();

        assert_eq!(
            row.fields(),
            vec![
                "1001",
                "",
                "1",
                "FAKE STREET",
                "TEST LOCALITY",
                "EXAMPLETON",
                "ZZ1 1ZZ",
                "ZZ11ZZ"
            ]
        );
    }

    #[test]
    fn rejects_wrong_field_count() {
        let err = PremisesRow::parse(&record(&["1", "2"])).unwrap_err();
        assert!(err.to_string().contains("expected 7 fields"), "{err}");
    }

    #[test]
    fn rejects_non_numeric_id() {
        let err = PremisesRow::parse(&record(&["abc", "", "", "", "", "", "ZZ1 1ZZ"])).unwrap_err();
        assert!(format!("{err:#}").contains("parsing id"), "{err:#}");
    }

    #[test]
    fn rejects_an_id_that_does_not_fit_the_column() {
        let err = PremisesRow::parse(&record(&["99999999999", "", "", "", "", "", "ZZ1 1ZZ"]))
            .unwrap_err();
        assert!(format!("{err:#}").contains("parsing id"), "{err:#}");
    }

    #[test]
    fn fields_line_up_with_declared_columns() {
        let row = PremisesRow::parse(&record(&["1", "a", "b", "c", "d", "e", "ZZ1 1ZZ"])).unwrap();
        assert_eq!(row.fields().len(), PremisesRow::COLUMNS.len());
    }
}
