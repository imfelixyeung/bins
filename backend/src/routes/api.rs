//! The API endpoints, mounted under `/api`.
//!
//! Query strings are read as a plain map and validated by hand so that every
//! failure leaves the body shape the Next.js routes returned: `error` plus
//! either `issues` or `message`. Two endpoints are the exception, answering
//! with something other than that envelope: `/api/jobs` with a CSV or a
//! calendar when one is asked for, and the sitemaps with XML, for crawlers.

pub mod jobs;
pub mod nearby;
pub mod premises;
pub mod random;
pub mod sitemaps;
pub mod status;

use std::collections::HashMap;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use sqlx::PgPool;
use tracing::error;

use crate::search::{NearbyPostcode, Premises};

/// The router for everything under `/api`, returning the state its handlers
/// need.
pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/api/jobs", get(jobs::handler))
        .route("/api/nearby", get(nearby::handler))
        .route("/api/premises", get(premises::handler))
        .route("/api/random/premises", get(random::handler))
        .route("/api/sitemaps/premises.xml", get(sitemaps::handler))
        .route("/api/status", get(status::handler))
}

/// Every successful response is wrapped in this envelope.
#[derive(Debug, Serialize)]
pub struct Envelope<T> {
    pub success: bool,
    pub timestamp: String,
    pub data: T,
}

impl<T> Envelope<T> {
    fn ok(data: T) -> Self {
        Self {
            success: true,
            timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            data,
        }
    }
}

/// A shorthand for the payload of `/api/premises`, where `data` is a list.
pub type PremisesBody = Json<Envelope<Vec<Premises>>>;

/// A shorthand for the payload of `/api/random/premises`, where `data` is one
/// premise, or `null` when there is nothing to choose from.
pub type RandomBody = Json<Envelope<Option<Premises>>>;

/// A shorthand for the payload of `/api/nearby`, where `data` is a list of
/// nearby postcodes, or `null` when the postcode asked for is not known.
pub type NearbyBody = Json<Envelope<Option<Vec<NearbyPostcode>>>>;

/// Anything that stops an endpoint from answering with `data`.
#[derive(Debug)]
pub enum ApiError {
    /// Every problem found with the query string.
    Invalid(Vec<Issue>),
    /// The premise asked for does not exist.
    NotFound(&'static str),
    /// Anything unexpected. Logged, never returned.
    Internal(anyhow::Error),
}

impl<E: Into<anyhow::Error>> From<E> for ApiError {
    fn from(error: E) -> Self {
        ApiError::Internal(error.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Invalid(issues) => (
                StatusCode::BAD_REQUEST,
                Json(Issues {
                    error: true,
                    issues,
                }),
            )
                .into_response(),
            ApiError::NotFound(message) => {
                (StatusCode::NOT_FOUND, Json(Message::new(message))).into_response()
            }
            ApiError::Internal(cause) => {
                error!(error = %cause, "request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(Message {
                        error: true,
                        message: "Internal server error".to_owned(),
                    }),
                )
                    .into_response()
            }
        }
    }
}

/// A rejected query parameter.
///
/// The `code`, `expected`, `received`, `options` and `path` fields exist because
/// clients of the Next.js API read them: they are the zod issues those routes
/// returned, reproduced field for field so this server is a drop-in
/// replacement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<&'static [&'static str]>,
    pub received: String,
    pub path: Vec<String>,
    pub message: String,
}

impl Issue {
    /// A parameter that has to be sent and was not.
    pub fn missing(path: &str, expected: &'static str) -> Self {
        Self {
            code: "invalid_type",
            expected: Some(expected),
            options: None,
            received: "undefined".to_owned(),
            path: vec![path.to_owned()],
            message: "Required".to_owned(),
        }
    }

    /// A parameter that is not a number. The legacy schema coerced whatever
    /// arrived, so an absent value was reported as `nan` too.
    pub fn not_a_number(path: &str) -> Self {
        Self {
            code: "invalid_type",
            expected: Some("number"),
            options: None,
            received: "nan".to_owned(),
            path: vec![path.to_owned()],
            message: "Expected number, received nan".to_owned(),
        }
    }

    /// A parameter outside the set of accepted values.
    pub fn not_an_option(path: &str, received: &str, options: &'static [&'static str]) -> Self {
        let accepted: Vec<String> = options.iter().map(|option| format!("'{option}'")).collect();

        Self {
            code: "invalid_enum_value",
            expected: None,
            options: Some(options),
            received: received.to_owned(),
            path: vec![path.to_owned()],
            message: format!(
                "Invalid enum value. Expected {}, received '{received}'",
                accepted.join(" | ")
            ),
        }
    }
}

#[derive(Debug, Serialize)]
struct Issues {
    error: bool,
    issues: Vec<Issue>,
}

#[derive(Debug, Serialize)]
struct Message {
    error: bool,
    message: String,
}

impl Message {
    fn new(message: impl Into<String>) -> Self {
        Self {
            error: true,
            message: message.into(),
        }
    }
}

/// The value of an optional query parameter, or `default`.
pub fn with_default<'a>(
    query: &'a HashMap<String, String>,
    key: &str,
    default: &'a str,
) -> &'a str {
    query.get(key).map_or(default, String::as_str)
}

/// `premises`, the id `/api/jobs` looks up.
pub fn premises_id(query: &HashMap<String, String>) -> Result<i32, Issue> {
    // Strictly an id, where zod coerced whatever arrived into a number. That
    // route answered `premises=42.4` with a 500 from Postgres, which is not worth
    // reproducing.
    query
        .get("premises")
        .ok_or_else(|| Issue::not_a_number("premises"))
        .and_then(|raw| {
            raw.trim()
                .parse()
                .map_err(|_| Issue::not_a_number("premises"))
        })
}

/// `postcode`, which `/api/premises` searches for.
pub fn postcode(query: &HashMap<String, String>) -> Result<&str, Issue> {
    query
        .get("postcode")
        .map(String::as_str)
        .ok_or_else(|| Issue::missing("postcode", "string"))
}

/// `page`, the paged sitemap `/api/sitemaps/premises.xml` serves. Absent asks
/// for the index of them all instead.
pub fn sitemap_page(query: &HashMap<String, String>) -> Result<Option<u32>, Issue> {
    query
        .get("page")
        .map(|raw| {
            // Read as a page index rather than an offset, so nothing a caller
            // sends can overflow the offset it is multiplied into.
            raw.trim().parse().map_err(|_| Issue::not_a_number("page"))
        })
        .transpose()
}

/// Checks a value against those accepted for its parameter.
pub fn one_of<'a>(
    value: &'a str,
    key: &str,
    options: &'static [&'static str],
) -> Result<&'a str, Issue> {
    options
        .contains(&value)
        .then_some(value)
        .ok_or_else(|| Issue::not_an_option(key, value, options))
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration};

    use super::*;

    fn query(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    fn json<T: Serialize>(value: &T) -> serde_json::Value {
        serde_json::to_value(value).expect("serialising")
    }

    #[test]
    fn an_absent_premises_id_is_reported_as_an_unreadable_number() {
        // The legacy schema coerced `undefined` to NaN rather than reporting a
        // missing parameter.
        assert_eq!(
            premises_id(&query(&[])).unwrap_err(),
            Issue::not_a_number("premises")
        );
    }

    #[test]
    fn a_non_numeric_premises_id_is_reported_as_an_issue() {
        let issue = Issue::not_a_number("premises");

        assert_eq!(issue.code, "invalid_type");
        assert_eq!(issue.expected, Some("number"));
        assert_eq!(issue.received, "nan");
        assert_eq!(issue.path, vec!["premises"]);
        assert_eq!(issue.message, "Expected number, received nan");

        assert_eq!(
            premises_id(&query(&[("premises", "abc")])).unwrap_err(),
            issue
        );
    }

    #[test]
    fn a_value_outside_the_options_is_reported_as_an_issue() {
        let issue = Issue::not_an_option("format", "xml", &["json", "csv", "ical"]);

        assert_eq!(
            json(&issue),
            serde_json::json!({
                "code": "invalid_enum_value",
                "options": ["json", "csv", "ical"],
                "received": "xml",
                "path": ["format"],
                "message": "Invalid enum value. Expected 'json' | 'csv' | 'ical', received 'xml'",
            })
        );

        assert_eq!(
            one_of("xml", "format", &["json", "csv", "ical"]).unwrap_err(),
            issue
        );
        assert_eq!(
            one_of("ical", "format", &["json", "csv", "ical"]).unwrap(),
            "ical"
        );
    }

    #[test]
    fn premises_ids_are_whole_numbers() {
        assert_eq!(premises_id(&query(&[("premises", "4242")])).unwrap(), 4242);
        assert_eq!(
            premises_id(&query(&[("premises", " 4242 ")])).unwrap(),
            4242
        );
        assert_eq!(premises_id(&query(&[("premises", "+4242")])).unwrap(), 4242);
        assert_eq!(
            premises_id(&query(&[("premises", "-4242")])).unwrap(),
            -4242
        );

        // Values the legacy coercion accepted but Postgres rejected with a 500.
        assert!(premises_id(&query(&[("premises", "42.4")])).is_err());
        assert!(premises_id(&query(&[("premises", "1e3")])).is_err());
        assert!(premises_id(&query(&[("premises", "0x10")])).is_err());
    }

    #[test]
    fn a_missing_postcode_is_reported_as_an_issue() {
        assert_eq!(
            postcode(&query(&[])).unwrap_err(),
            Issue::missing("postcode", "string")
        );
    }

    #[test]
    fn postcodes_are_taken_as_given() {
        assert_eq!(
            postcode(&query(&[("postcode", "LS6 2SE")])).unwrap(),
            "LS6 2SE"
        );
        // Normalisation happens in the query, not here.
        assert_eq!(
            postcode(&query(&[("postcode", "ls62se")])).unwrap(),
            "ls62se"
        );
        assert_eq!(postcode(&query(&[("postcode", "")])).unwrap(), "");
    }

    #[test]
    fn an_absent_sitemap_page_asks_for_the_index() {
        assert_eq!(sitemap_page(&query(&[])).unwrap(), None);
        assert_eq!(sitemap_page(&query(&[("page", "0")])).unwrap(), Some(0));
        assert_eq!(sitemap_page(&query(&[("page", " 8 ")])).unwrap(), Some(8));
    }

    #[test]
    fn a_sitemap_page_that_is_not_a_whole_number_is_reported_as_an_issue() {
        // The schema this replaced read the page off the path, so the only
        // numbers that matter are the ones a page index could be.
        for page in [
            "abc",
            "",
            "-1",
            "1.5",
            "1e3",
            "0x10",
            "99999999999999999999",
        ] {
            assert_eq!(
                sitemap_page(&query(&[("page", page)])).unwrap_err(),
                Issue::not_a_number("page"),
                "page={page:?} should have been rejected"
            );
        }
    }

    #[test]
    fn optional_parameters_fall_back_to_their_default() {
        assert_eq!(with_default(&query(&[]), "format", "json"), "json");
        assert_eq!(
            with_default(&query(&[("format", "csv")]), "format", "json"),
            "csv"
        );
        // An empty value is a value, so it is checked rather than defaulted.
        assert_eq!(
            with_default(&query(&[("format", "")]), "format", "json"),
            ""
        );
    }

    #[test]
    fn an_envelope_carries_the_generated_timestamp() {
        let before = Utc::now();
        let body = json(&Envelope::ok(1));
        let after = Utc::now();

        assert_eq!(body["success"], true);
        assert_eq!(body["data"], 1);

        let timestamp = body["timestamp"].as_str().expect("timestamp");
        // Formatted to millisecond precision, so the value can sit just under
        // `before`.
        let slack = Duration::milliseconds(1);
        let parsed = DateTime::parse_from_rfc3339(timestamp)
            .expect("RFC 3339 timestamp")
            .with_timezone(&Utc);
        assert!(
            parsed + slack >= before && parsed <= after,
            "timestamp {timestamp} is not the time of response"
        );
        // Millisecond precision, as `Date.prototype.toISOString` produced.
        assert!(
            timestamp.ends_with('Z') && timestamp.len() == 24,
            "{timestamp}"
        );
    }

    #[test]
    fn error_bodies_match_the_documented_shape() {
        let message = json(&Message::new("Premises not found"));
        assert_eq!(
            message,
            serde_json::json!({ "error": true, "message": "Premises not found" })
        );

        let issues = json(&Issues {
            error: true,
            issues: vec![Issue::missing("premises", "number")],
        });
        assert_eq!(
            issues,
            serde_json::json!({
                "error": true,
                "issues": [{
                    "code": "invalid_type",
                    "expected": "number",
                    "received": "undefined",
                    "path": ["premises"],
                    "message": "Required",
                }]
            })
        );
    }
}
