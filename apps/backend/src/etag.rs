//! What upstream says about the files it publishes, remembered in the `etags`
//! table so a sync can tell whether the file it is about to download is the one
//! it already holds.
//!
//! Ported from `apps/worker/etag.ts`, which did the same thing in front of the
//! import scripts. Leeds sends both an `etag` and a `last-modified` header for
//! each CSV, and only the etag decides whether an import runs: a file whose
//! etag has not moved is not downloaded, which is what keeps a daily sync of a
//! 170MB feed cheap.
//!
//! The etag is written once the import has succeeded, never before, so a run
//! that fails is retried on the next tick instead of being written off as done.

use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use reqwest::header::{ETAG, LAST_MODIFIED};
use sqlx::{FromRow, PgPool};
use tokio::time::sleep;
use tracing::warn;

/// How long one request to upstream may take, which is the connect timeout the
/// Next.js worker allowed.
const TIMEOUT_SECONDS: u64 = 60;

/// How many times a request is made before the check gives up, which is the
/// four attempts `p-retry` made there: the first, and three retries.
const ATTEMPTS: usize = 4;

/// How long to wait between attempts, doubling each time so a host that is
/// struggling is not hammered.
const BACKOFF: Duration = Duration::from_millis(500);

/// What upstream says about a file at the moment it was asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Latest {
    /// The `etag` header, stored and compared exactly as it was sent, quotes
    /// and all.
    pub etag: String,
    /// The `last-modified` header, which is when the file itself last changed.
    pub modified_at: DateTime<Utc>,
    /// When the file was asked about, which moves on whether anything changed
    /// or not.
    pub checked_at: DateTime<Utc>,
}

/// What the database remembers about a file.
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Stored {
    /// `NULL` for a file that has been checked but never imported, which no
    /// file is in practice: the etag is only written after an import.
    pub etag: Option<String>,
    pub modified_at: Option<DateTime<Utc>>,
    /// When the file was last asked about.
    pub checked_at: DateTime<Utc>,
}

/// A file compared against what is already stored, which is what a sync acts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// `None` when the file has never been imported, in which case there is
    /// nothing to compare and the import runs.
    pub stored: Option<Stored>,
    pub latest: Latest,
}

/// Whether `latest` is the same version of the file as the one stored.
///
/// Only the etag decides this, as it did in the Next.js worker: the
/// `last-modified` header is kept for display but is not compared.
pub fn unchanged(stored: Option<&Stored>, latest: &Latest) -> bool {
    stored.and_then(|stored| stored.etag.as_deref()) == Some(latest.etag.as_str())
}

/// What is stored against what upstream says now.
pub async fn check(pool: &PgPool, url: &str) -> Result<Check> {
    Ok(Check {
        stored: stored(pool, url).await?,
        latest: fetch(url).await?,
    })
}

/// What the database holds for `url`, if it holds anything at all.
pub async fn stored(pool: &PgPool, url: &str) -> Result<Option<Stored>> {
    sqlx::query_as("SELECT etag, modified_at, checked_at FROM etags WHERE url = $1")
        .bind(url)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("reading the etag stored for {url}"))
}

/// Records that `latest` is now the version of the file the tables hold.
///
/// Called once the import has succeeded, so a run that fails leaves the stored
/// etag where it was and the next run imports again.
pub async fn store(pool: &PgPool, url: &str, latest: &Latest) -> Result<()> {
    sqlx::query(
        "INSERT INTO etags (url, etag, modified_at, checked_at) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (url) DO UPDATE SET \
             etag = excluded.etag, \
             modified_at = excluded.modified_at, \
             checked_at = excluded.checked_at",
    )
    .bind(url)
    .bind(&latest.etag)
    .bind(latest.modified_at)
    .bind(latest.checked_at)
    .execute(pool)
    .await
    .with_context(|| format!("storing the etag of {url}"))?;

    Ok(())
}

/// Records that upstream was asked about `url`, when nothing had changed.
///
/// Only the time of the check moves on: the etag is the same one that is
/// already stored, and the row is there because that is what was compared. The
/// Next.js worker rewrote the whole record instead, which made no difference to
/// what was read back.
pub async fn checked(pool: &PgPool, url: &str, latest: &Latest) -> Result<()> {
    sqlx::query("UPDATE etags SET checked_at = $2 WHERE url = $1")
        .bind(url)
        .bind(latest.checked_at)
        .execute(pool)
        .await
        .with_context(|| format!("recording the etag check of {url}"))?;

    Ok(())
}

/// Asks upstream for the etag of `url`, retrying a few times before giving up.
pub async fn fetch(url: &str) -> Result<Latest> {
    let mut wait = BACKOFF;

    for attempt in 1..=ATTEMPTS {
        let error = match request(url).await {
            Ok(latest) => return Ok(latest),
            Err(error) => error,
        };

        if attempt == ATTEMPTS {
            return Err(error);
        }

        warn!(%url, attempt, error = %error, "retrying the etag request");
        sleep(wait).await;
        wait *= 2;
    }

    // Only reached when `ATTEMPTS` is zero, which `fetch` never runs with.
    Err(anyhow!("the etag of {url} was never requested"))
}

/// One `HEAD` request, which upstream answers with the headers describing the
/// file rather than the file itself.
///
/// Both headers are required, as they were in the Next.js worker: without them
/// there is no way to tell what changed, or when, so a file that sends neither
/// is treated as one that cannot be checked.
async fn request(url: &str) -> Result<Latest> {
    // Taken before the request rather than after, so a slow response is not
    // counted as checked before it was.
    let checked_at = Utc::now();

    let response = reqwest::Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECONDS))
        .build()
        .context("building an HTTP client")?
        .head(url)
        .send()
        .await
        .with_context(|| format!("requesting {url}"))?
        .error_for_status()
        .with_context(|| format!("requesting {url}"))?;

    let headers = response.headers();

    let etag = header(headers, ETAG)
        .with_context(|| format!("{url} sent no etag header"))?
        .to_owned();
    let modified = header(headers, LAST_MODIFIED)
        .with_context(|| format!("{url} sent no last-modified header"))?;

    Ok(Latest {
        etag,
        modified_at: parse_http_date(modified)
            .with_context(|| format!("reading the last-modified header of {url}"))?,
        checked_at,
    })
}

fn header(headers: &reqwest::header::HeaderMap, name: reqwest::header::HeaderName) -> Option<&str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

/// Reads an HTTP date, the format `last-modified` is written in.
///
/// RFC 9110 spells it three ways and a sender may use any of them, so it is
/// read by a parser written for the format rather than by one for RFC 2822,
/// which rejects two of the three.
fn parse_http_date(raw: &str) -> Result<DateTime<Utc>> {
    httpdate::parse_http_date(raw.trim())
        .map(DateTime::from)
        .with_context(|| format!("{raw:?} is not an HTTP date"))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn latest(etag: &str) -> Latest {
        Latest {
            etag: etag.to_owned(),
            modified_at: Utc.with_ymd_and_hms(2026, 10, 5, 3, 45, 41).unwrap(),
            checked_at: Utc.with_ymd_and_hms(2026, 10, 5, 4, 0, 0).unwrap(),
        }
    }

    fn stored(etag: Option<&str>) -> Stored {
        Stored {
            etag: etag.map(str::to_owned),
            modified_at: Some(Utc.with_ymd_and_hms(2026, 10, 4, 3, 45, 41).unwrap()),
            checked_at: Utc.with_ymd_and_hms(2026, 10, 5, 4, 0, 0).unwrap(),
        }
    }

    #[test]
    fn the_same_etag_is_unchanged() {
        // The header arrives quoted, and is stored exactly as it arrived.
        assert!(unchanged(
            Some(&stored(Some("\"f01141fe7b54dd1:0\""))),
            &latest("\"f01141fe7b54dd1:0\"")
        ));
    }

    #[test]
    fn a_different_etag_is_a_change() {
        assert!(!unchanged(
            Some(&stored(Some("\"f01141fe7b54dd1:0\""))),
            &latest("\"a1b2c3d4e5f6:0\"")
        ));
    }

    #[test]
    fn a_file_that_has_never_been_imported_is_a_change() {
        assert!(!unchanged(None, &latest("\"f01141fe7b54dd1:0\"")));
        // An imported file whose etag was lost is treated the same way, since
        // there is nothing to compare against.
        assert!(!unchanged(
            Some(&stored(None)),
            &latest("\"f01141fe7b54dd1:0\"")
        ));
    }

    #[test]
    fn an_etag_that_differs_only_in_whitespace_is_a_change() {
        // Upstream quotes its etag and the quotes are part of the value, so a
        // missing quote is a different etag rather than the same one.
        assert!(!unchanged(
            Some(&stored(Some("f01141fe7b54dd1:0"))),
            &latest("\"f01141fe7b54dd1:0\"")
        ));
    }

    #[test]
    fn reads_the_date_an_http_header_carries() {
        let expected = Utc.with_ymd_and_hms(2026, 10, 5, 3, 45, 41).unwrap();

        // RFC 9110 allows three spellings, and a sender may use any of them. The
        // first is the one the open data host sends.
        for raw in [
            "Mon, 05 Oct 2026 03:45:41 GMT",
            "Monday, 05-Oct-26 03:45:41 GMT",
            "Mon Oct  5 03:45:41 2026",
            "  Mon, 05 Oct 2026 03:45:41 GMT  ",
        ] {
            assert_eq!(parse_http_date(raw).unwrap(), expected, "{raw:?}");
        }
    }

    #[test]
    fn reads_a_date_in_the_far_past() {
        // The two digit year of the older spellings is read as the year it stands
        // for, so a header long past 1970 does not become a date after the
        // epoch.
        assert_eq!(
            parse_http_date("Thursday, 01-Jan-70 00:00:00 GMT").unwrap(),
            Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn rejects_a_header_that_is_not_a_date() {
        // The zone is part of the format, so a date without one is not a date,
        // and neither is one carrying a zone other than GMT.
        for raw in [
            "",
            "not a date",
            "Mon, 05 Oct 2026 03:45:41",
            "Mon, 05 Oct 2026 03:45:41 +01:00",
            "Mon, 32 Oct 2026 03:45:41 GMT",
        ] {
            assert!(parse_http_date(raw).is_err(), "{raw:?}");
        }
    }
}
