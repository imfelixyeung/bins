//! Coordinates for postcodes, fetched from postcodes.io and stored so the API
//! can answer "what is near here" without calling out to anyone.
//!
//! The `postcodes` table already has a generated `geography` column with a GiST
//! index, so this module only has to keep it populated. The Next.js worker did
//! the same work in `apps/worker/postcodes.ts`: every run it picked one
//! postcode that had no coordinates yet, asked postcodes.io for that postcode
//! and its 100 nearest neighbours within 2km, and stored all of them. The
//! neighbours come along for free, so each run fills in a hundred postcodes at a
//! time and the table crawls towards covering every premise.

use anyhow::{Context, Result};
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::{AssertSqlSafe, PgPool};
use tracing::{info, warn};

/// Where postcodes.io lives, overridable so the sync can be pointed at a stub.
const API_URL: &str = "https://api.postcodes.io";

/// How many postcodes to ask for, and how far away they may be. Both match what
/// the Next.js worker asked for, so the table fills at the same rate.
const LIMIT: usize = 100;
const RADIUS_METRES: f64 = 2_000.0;

/// How long one request to postcodes.io may take. Cron has no supervisor, so a
/// hung request would otherwise leave the job running until the container is
/// restarted.
const TIMEOUT_SECONDS: u64 = 30;

/// The form postcodes.io sends and the `postcodes` table is keyed by: uppercase,
/// with a space before the inward code.
///
/// Callers may send `LS62SE`, `ls6 2se` or ` LS6 2SE `, and because the lookup is
/// an exact match on the primary key they all have to arrive here as the same
/// string.
pub fn canonical(raw: &str) -> String {
    let chars: Vec<char> = raw
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    // Anything too short to hold an inward code is left alone rather than
    // mangled into a leading space.
    if chars.len() <= 3 {
        return chars.into_iter().collect();
    }

    let split = chars.len() - 3;
    let (outward, inward) = chars.split_at(split);

    let mut canonical: String = outward.iter().collect();
    canonical.push(' ');
    canonical.extend(inward);
    canonical
}

/// One postcode and where it is. postcodes.io sends around fifty fields per
/// postcode; only these three are kept.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Nearby {
    pub postcode: String,
    pub latitude: f64,
    pub longitude: f64,
}

/// The response wrapper postcodes.io sends.
#[derive(Debug, Deserialize)]
struct Response {
    result: Vec<Nearby>,
}

/// One run of the sync, reporting the postcode it worked on so the log says
/// which hole is being filled.
pub async fn sync(pool: &PgPool) -> Result<Option<String>> {
    let Some(postcode) = missing_postcode(pool).await? else {
        return Ok(None);
    };

    info!(%postcode, "fetching postcode data");

    let nearby = fetch_nearest(&postcode).await?;

    // A postcode with nothing within 2km is not an error: postcodes.io simply
    // knows of no neighbours, and the next run will try again.
    if nearby.is_empty() {
        warn!(%postcode, "postcodes.io knows of no nearby postcodes");
        return Ok(Some(postcode));
    }

    let stored = upsert_nearest(pool, &nearby).await?;

    info!(%postcode, nearby = nearby.len(), stored, "postcode data stored");

    Ok(Some(postcode))
}

/// A postcode that has premises but no coordinates yet.
///
/// Picking at random keeps the table filling evenly from one end of the country
/// to the other, which is what the Next.js worker did with `ORDER BY RANDOM()`.
pub async fn missing_postcode(pool: &PgPool) -> Result<Option<String>> {
    let sql = AssertSqlSafe(
        "SELECT premises.address_postcode FROM dm_premises premises \
         LEFT JOIN postcodes ON premises.address_postcode = postcodes.id \
         WHERE premises.address_postcode IS NOT NULL AND postcodes.id IS NULL \
         ORDER BY random() LIMIT 1",
    );

    let postcode: Option<(String,)> = sqlx::query_as(sql)
        .fetch_optional(pool)
        .await
        .context("finding a postcode without coordinates")?;

    Ok(postcode.map(|(postcode,)| postcode))
}

/// The `postcode` and its [`LIMIT`] nearest neighbours within [`RADIUS_METRES`].
///
/// An unknown postcode is not an error: postcodes.io answers `404` for those and
/// the caller stores nothing.
pub async fn fetch_nearest(postcode: &str) -> Result<Vec<Nearby>> {
    let base = std::env::var("POSTCODES_API_URL").unwrap_or_else(|_| API_URL.to_owned());
    let url =
        format!("{base}/postcodes/{postcode}/nearest?limit={LIMIT}&radius={RADIUS_METRES:.0}");

    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECONDS))
        .build()
        .context("building an HTTP client")?
        .get(&url)
        .send()
        .await
        .with_context(|| format!("requesting {url}"))?;

    if response.status() == StatusCode::NOT_FOUND {
        warn!(%postcode, "postcode not found");
        return Ok(Vec::new());
    }

    let response = response
        .error_for_status()
        .with_context(|| format!("requesting {url}"))?
        .json::<Response>()
        .await
        .with_context(|| format!("reading {url}"))?;

    Ok(response.result)
}

/// Stores coordinates, overwriting any that were there before.
///
/// `location` is a generated column, so it follows the coordinates and needs no
/// attention here, and the table's trigger keeps `updated_at` honest.
pub async fn upsert_nearest(pool: &PgPool, nearby: &[Nearby]) -> Result<u64> {
    if nearby.is_empty() {
        return Ok(0);
    }

    let postcodes: Vec<&str> = nearby.iter().map(|p| p.postcode.as_str()).collect();
    let latitudes: Vec<f64> = nearby.iter().map(|p| p.latitude).collect();
    let longitudes: Vec<f64> = nearby.iter().map(|p| p.longitude).collect();

    let sql = AssertSqlSafe(
        "INSERT INTO postcodes (id, latitude, longitude) \
         SELECT * FROM UNNEST($1::text[], $2::double precision[], $3::double precision[]) \
         ON CONFLICT (id) DO UPDATE SET \
             latitude = excluded.latitude, longitude = excluded.longitude",
    );

    let stored = sqlx::query(sql)
        .bind(&postcodes)
        .bind(&latitudes)
        .bind(&longitudes)
        .execute(pool)
        .await
        .context("storing postcode coordinates")?
        .rows_affected();

    Ok(stored)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(body: &str) -> Result<Vec<Nearby>> {
        Ok(serde_json::from_str::<Response>(body)?.result)
    }

    #[test]
    fn reads_the_postcode_and_its_coordinates() {
        let nearby = parse(
            r#"{
                "status": 200,
                "result": [
                    { "postcode": "LS6 2SE", "latitude": 53.821771, "longitude": -1.563046, "distance": 0 },
                    { "postcode": "LS6 2QF", "latitude": 53.8, "longitude": -1.5, "distance": 12.34 }
                ]
            }"#,
        )
        .expect("a postcodes.io response");

        assert_eq!(nearby.len(), 2);
        assert_eq!(nearby[0].postcode, "LS6 2SE");
        assert_eq!(nearby[0].latitude, 53.821771);
        assert_eq!(nearby[0].longitude, -1.563046);
        assert_eq!(nearby[1].postcode, "LS6 2QF");
    }

    #[test]
    fn a_postcode_with_no_neighbours_is_an_empty_list() {
        let nearby = parse(r#"{ "status": 200, "result": [] }"#).expect("a response");

        assert!(nearby.is_empty());
    }

    #[test]
    fn rejects_a_result_that_is_missing_a_coordinate() {
        // The Next.js worker parsed this with zod and threw, which failed the
        // run and left the postcode for a later attempt.
        let error = parse(r#"{ "result": [{ "postcode": "LS6 2SE", "latitude": 53.8 }] }"#)
            .expect_err("a response without a longitude");

        assert!(error.to_string().contains("longitude"), "{error}");
    }

    #[test]
    fn rejects_a_body_that_is_not_the_expected_shape() {
        assert!(parse(r#"{ "status": 404, "error": "Invalid postcode" }"#).is_err());
        assert!(parse("not json").is_err());
    }

    #[test]
    fn postcodes_are_keyed_the_way_postcodes_io_sends_them() {
        assert_eq!(canonical("LS62SE"), "LS6 2SE");
        assert_eq!(canonical("ls6 2se"), "LS6 2SE");
        assert_eq!(canonical(" LS6\t2SE\n"), "LS6 2SE");
    }

    #[test]
    fn postcodes_without_an_inward_code_are_left_alone() {
        assert_eq!(canonical("LS6"), "LS6");
        assert_eq!(canonical(""), "");
        assert_eq!(canonical("  "), "");
    }
}
