//! `GET /api/jobs`: the collections scheduled for one premise.

use std::collections::HashMap;

use axum::Json;
use axum::extract::{Query, State};
use sqlx::PgPool;

use crate::search;

use super::{ApiError, Envelope, Issue, JobsBody, one_of, premises_id, with_default};

/// The formats the endpoint documents. Only `json` has been ported so far; the
/// others used to be produced by CSV and iCal writers next to the Next.js route.
const FORMATS: [&str; 3] = ["json", "csv", "ical"];

/// `GET /api/jobs?premises=<id>[&format=json]`
pub async fn handler(
    State(pool): State<PgPool>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<JobsBody, ApiError> {
    let (premises, format) = parse(&query).map_err(ApiError::Invalid)?;

    if format != "json" {
        return Err(ApiError::Unsupported(format!(
            "format {format:?} is not supported yet"
        )));
    }

    let data = search::jobs(&pool, premises)
        .await?
        .ok_or(ApiError::NotFound("Premises not found"))?;

    Ok(Json(Envelope::ok(data)))
}

/// Reads the query string, checking every parameter so the response lists every
/// problem at once rather than only the first.
fn parse(query: &HashMap<String, String>) -> Result<(i32, &str), Vec<Issue>> {
    let premises = premises_id(query);
    let format = one_of(with_default(query, "format", "json"), "format", &FORMATS);

    let issues: Vec<Issue> = [
        premises.as_ref().err().cloned(),
        format.as_ref().err().cloned(),
    ]
    .into_iter()
    .flatten()
    .collect();

    if let (Ok(premises), Ok(format)) = (premises, format) {
        return Ok((premises, format));
    }

    Err(issues)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    #[test]
    fn json_is_the_default_format() {
        assert_eq!(parse(&query(&[("premises", "1")])).unwrap().1, "json");
        assert_eq!(
            parse(&query(&[("premises", "1"), ("format", "json")]))
                .unwrap()
                .1,
            "json"
        );
    }

    #[test]
    fn reads_the_premises_id() {
        assert_eq!(parse(&query(&[("premises", "4242")])).unwrap().0, 4242);
    }

    #[test]
    fn reports_every_problem_at_once() {
        let issues = parse(&query(&[("premises", "abc"), ("format", "xml")])).unwrap_err();

        assert_eq!(
            issues,
            vec![
                Issue::not_a_number("premises"),
                Issue::not_an_option("format", "xml", &FORMATS),
            ]
        );
    }
}
