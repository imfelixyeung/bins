//! The reads behind the `/api/premises`, `/api/random/premises` and `/api/jobs`
//! endpoints.
//!
//! The row structs double as the API response shape, so the field names and
//! ordering here are what clients see and must not drift.

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use serde::{Serialize, Serializer};
use sqlx::{AssertSqlSafe, FromRow, PgPool};

use crate::import::premises::search_postcode;

/// The address columns both endpoints return. `search_postcode` is only the
/// lookup key and `created_at` is never served, so neither is selected.
const PREMISES_COLUMNS: &str = "id, address_room, address_number, address_street, \
                               address_locality, address_city, address_postcode, updated_at";

/// Past this many matches the addresses are left in database order, because
/// sorting them costs more than the ordering is worth for a postcode holding
/// hundreds of properties.
const MAX_SORTED_RESULTS: usize = 100;

/// An address as the API serves it. `address_room` and friends are free text
/// from the upstream feed, so all but `id` are optional.
#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Premises {
    pub id: i32,
    pub address_room: Option<String>,
    pub address_number: Option<String>,
    pub address_street: Option<String>,
    pub address_locality: Option<String>,
    pub address_city: Option<String>,
    pub address_postcode: Option<String>,
    #[serde(serialize_with = "serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

/// One scheduled collection.
#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct Job {
    pub bin: String,
    /// `NaiveDate` serialises as `YYYY-MM-DD`, which is what clients expect.
    pub date: NaiveDate,
}

/// A premise together with its upcoming jobs, as served by `/api/jobs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PremisesJobs {
    #[serde(flatten)]
    pub premises: Premises,
    pub jobs: Vec<Job>,
}

/// Timestamps go out in the same shape `Date.prototype.toISOString` produced in
/// the Next.js app: UTC, always with millisecond precision.
fn serialize_timestamp<S: Serializer>(
    timestamp: &DateTime<Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&timestamp.to_rfc3339_opts(SecondsFormat::Millis, true))
}

/// Every premise at `postcode`, ordered by house number.
pub async fn premises(pool: &PgPool, postcode: &str) -> Result<Vec<Premises>> {
    let key = search_postcode(postcode);

    let sql = AssertSqlSafe(format!(
        "SELECT {PREMISES_COLUMNS} FROM dm_premises WHERE search_postcode = $1 \
         ORDER BY address_room, address_number, address_street, id"
    ));

    let mut found: Vec<Premises> = sqlx::query_as(sql)
        .bind(&key)
        .fetch_all(pool)
        .await
        .with_context(|| format!("querying premises for postcode {key}"))?;

    // Addresses in the feed sort as text ("10" before "9"), so a result set small
    // enough to sort is re-ordered numerically. `sort_by_key` is stable, which
    // keeps the database ordering for everything that ties.
    if found.len() <= MAX_SORTED_RESULTS {
        found.sort_by_key(|premise| address_number_sorter(&premise.address_number));
    }

    Ok(found)
}

/// A premise chosen at random, or `None` when there are no premises at all.
///
/// `ORDER BY random()` has no index to work from, so this reads every row. The
/// Next.js app did the same, behind a one second cache.
pub async fn random_premise(pool: &PgPool) -> Result<Option<Premises>> {
    let sql = AssertSqlSafe(format!(
        "SELECT {PREMISES_COLUMNS} FROM dm_premises ORDER BY random() LIMIT 1"
    ));

    sqlx::query_as(sql)
        .fetch_optional(pool)
        .await
        .context("querying a random premise")
}

/// The premise with `premises_id` and its jobs, or `None` when no such premise
/// exists.
pub async fn jobs(pool: &PgPool, premises_id: i32) -> Result<Option<PremisesJobs>> {
    let sql = AssertSqlSafe(format!(
        "SELECT {PREMISES_COLUMNS} FROM dm_premises WHERE id = $1"
    ));

    let premises: Option<Premises> = sqlx::query_as(sql)
        .bind(premises_id)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("querying premise {premises_id}"))?;

    let Some(premises) = premises else {
        return Ok(None);
    };

    let jobs: Vec<Job> = sqlx::query_as(
        "SELECT bin, date FROM dm_jobs WHERE premises_id = $1 ORDER BY date, bin, id",
    )
    .bind(premises_id)
    .fetch_all(pool)
    .await
    .with_context(|| format!("querying jobs for premise {premises_id}"))?;

    Ok(Some(PremisesJobs { premises, jobs }))
}

/// The house number to sort on. Anything that does not start with digits sorts
/// as zero, which puts unnumbered addresses ahead of numbered ones.
fn address_number_sorter(address_number: &Option<String>) -> i64 {
    address_number
        .as_deref()
        .and_then(parse_leading_int)
        .unwrap_or(0)
}

/// Reads the leading run of digits, ignoring surrounding whitespace and an
/// optional sign, as `parseInt` did. Values that are not a whole number, or
/// that do not fit an `i64`, yield `None`.
fn parse_leading_int(raw: &str) -> Option<i64> {
    let trimmed = raw.trim_start();
    let bytes = trimmed.as_bytes();
    let mut end = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let digits = end;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end == digits {
        return None;
    }

    trimmed[..end].parse().ok()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn premise(id: i32, address_number: Option<&str>) -> Premises {
        Premises {
            id,
            address_room: None,
            address_number: address_number.map(str::to_owned),
            address_street: Some("FAKE STREET".to_owned()),
            address_locality: None,
            address_city: Some("LEEDS".to_owned()),
            address_postcode: Some("ZZ1 1ZZ".to_owned()),
            updated_at: Utc.with_ymd_and_hms(2026, 10, 3, 2, 1, 5).unwrap(),
        }
    }

    fn json<T: Serialize>(value: &T) -> serde_json::Value {
        serde_json::to_value(value).expect("serialising")
    }

    #[test]
    fn reads_leading_digits_only() {
        assert_eq!(parse_leading_int("12"), Some(12));
        assert_eq!(parse_leading_int("12A"), Some(12));
        assert_eq!(parse_leading_int("12 14"), Some(12));
        assert_eq!(parse_leading_int(" 12"), Some(12));
        assert_eq!(parse_leading_int("-3"), Some(-3));
        assert_eq!(parse_leading_int("+3"), Some(3));
    }

    #[test]
    fn rejects_values_without_leading_digits() {
        assert_eq!(parse_leading_int(""), None);
        assert_eq!(parse_leading_int("   "), None);
        assert_eq!(parse_leading_int("A12"), None);
        assert_eq!(parse_leading_int("-"), None);
        assert_eq!(parse_leading_int("99999999999999999999"), None);
    }

    #[test]
    fn missing_or_unnumbered_addresses_sort_as_zero() {
        assert_eq!(address_number_sorter(&None), 0);
        assert_eq!(address_number_sorter(&Some("FLAT 2".to_owned())), 0);
        assert_eq!(address_number_sorter(&Some("10".to_owned())), 10);
    }

    #[test]
    fn sorts_house_numbers_numerically() {
        let mut found = [
            premise(1, Some("10")),
            premise(2, Some("9")),
            premise(3, Some("100")),
        ];
        found.sort_by_key(|premise| address_number_sorter(&premise.address_number));

        let ids: Vec<i32> = found.iter().map(|premise| premise.id).collect();
        assert_eq!(ids, vec![2, 1, 3]);
    }

    #[test]
    fn unnumbered_addresses_keep_their_database_order() {
        let mut found = [
            premise(1, Some("FLAT 1")),
            premise(2, Some("FLAT 2")),
            premise(3, Some("2")),
        ];
        found.sort_by_key(|premise| address_number_sorter(&premise.address_number));

        let ids: Vec<i32> = found.iter().map(|premise| premise.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn serialises_address_fields_as_camel_case() {
        let json = json(&premise(4242, Some("6")));

        assert_eq!(json["id"], 4242);
        assert_eq!(json["addressRoom"], serde_json::Value::Null);
        assert_eq!(json["addressNumber"], "6");
        assert_eq!(json["addressStreet"], "FAKE STREET");
        assert_eq!(json["addressLocality"], serde_json::Value::Null);
        assert_eq!(json["addressCity"], "LEEDS");
        assert_eq!(json["addressPostcode"], "ZZ1 1ZZ");
        assert_eq!(
            json.as_object().expect("object").len(),
            8,
            "only the selected columns should be served"
        );
    }

    #[test]
    fn timestamps_are_utc_with_millisecond_precision() {
        let json = json(&premise(1, None));

        assert_eq!(json["updatedAt"], "2026-10-03T02:01:05.000Z");
    }

    #[test]
    fn jobs_serialise_dates_as_iso_days() {
        let json = json(&Job {
            bin: "GREEN".to_owned(),
            date: NaiveDate::from_ymd_opt(2026, 11, 2).unwrap(),
        });

        assert_eq!(
            json,
            serde_json::json!({ "bin": "GREEN", "date": "2026-11-02" })
        );
    }

    #[test]
    fn jobs_are_nested_under_the_address() {
        let found = PremisesJobs {
            premises: premise(4242, Some("6")),
            jobs: vec![Job {
                bin: "BLACK".to_owned(),
                date: NaiveDate::from_ymd_opt(2026, 11, 2).unwrap(),
            }],
        };

        let json = json(&found);
        assert_eq!(json["id"], 4242);
        assert_eq!(json["addressPostcode"], "ZZ1 1ZZ");
        assert_eq!(
            json["jobs"],
            serde_json::json!([{ "bin": "BLACK", "date": "2026-11-02" }])
        );
    }
}
