//! The reads and XML behind `/api/sitemaps/premises.xml`.
//!
//! One request is either the index of every paged sitemap, or one of those pages
//! of premise URLs.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::search::format_timestamp;

/// How many premises each paged sitemap holds. 50,000 is the most URLs a
/// sitemap may contain.
pub const PAGE_SIZE: u32 = 50_000;

/// Where these sitemaps are published. The proxy puts this server behind
/// `/api` and strips the version again, so the URLs written into a sitemap
/// have to carry the prefix a crawler will ask for, not the path served here.
const PUBLIC_PREFIX: &str = "/api";

/// The site the sitemaps describe when `BASE_URL` is not set.
const DEFAULT_SITE: &str = "https://bins.felixyeung.com";

/// The sitemap namespace every document here is written in.
const NAMESPACE: &str = "http://www.sitemaps.org/schemas/sitemap/0.9";

/// One premise, which is all a sitemap entry needs from it.
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Premise {
    pub id: i32,
    pub updated_at: DateTime<Utc>,
}

/// The site whose premises these sitemaps list.
pub fn site() -> String {
    normalised(&std::env::var("BASE_URL").unwrap_or_else(|_| DEFAULT_SITE.to_owned())).to_owned()
}

/// A site is a host and nothing after it, so that a trailing slash cannot leave
/// the URLs below with a doubled one.
fn normalised(site: &str) -> &str {
    site.trim_end_matches('/')
}

/// Whether the app was started with `SKIP_SITEMAP` set, which leaves the
/// sitemaps alone.
pub fn skipped() -> bool {
    skipped_by(std::env::var("SKIP_SITEMAP").ok())
}

fn skipped_by(value: Option<String>) -> bool {
    // An empty value is how compose spells "not skipping".
    value.is_some_and(|set| !set.is_empty())
}

/// How many paged sitemaps cover every premise. Zero when generation is
/// skipped.
pub async fn pages(pool: &PgPool) -> Result<usize> {
    if skipped() {
        return Ok(0);
    }

    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM dm_premises")
        .fetch_one(pool)
        .await
        .context("counting premises")?;

    // Rounded up, so a count that lands on a page boundary does not add an
    // empty one.
    Ok(count.div_euclid(i64::from(PAGE_SIZE)) as usize
        + usize::from(count.rem_euclid(i64::from(PAGE_SIZE)) > 0))
}

/// One page of premises, oldest id first. Empty past the last page, and when
/// generation is skipped.
pub async fn page(pool: &PgPool, page: u32) -> Result<Vec<Premise>> {
    if skipped() {
        return Ok(Vec::new());
    }

    let offset = i64::from(page) * i64::from(PAGE_SIZE);

    let premises =
        sqlx::query_as("SELECT id, updated_at FROM dm_premises ORDER BY id LIMIT $1 OFFSET $2")
            .bind(i64::from(PAGE_SIZE))
            .bind(offset)
            .fetch_all(pool)
            .await
            .with_context(|| format!("reading page {page} of the premises sitemap"))?;

    Ok(premises)
}

/// The index of paged sitemaps, one `<sitemap>` per page.
pub fn index(site: &str, pages: usize) -> String {
    let site = normalised(site);

    let sitemaps = (0..pages)
        .map(|page| {
            format!(
                "  <sitemap>\n    <loc>{}</loc>\n  </sitemap>",
                escape(&page_url(site, page))
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    document("sitemapindex", &sitemaps)
}

/// One paged sitemap: the premises on it, with the address pages they are at.
pub fn urlset(site: &str, premises: &[Premise]) -> String {
    let site = normalised(site);

    let urls = premises
        .iter()
        .map(|premise| {
            format!(
                "  <url>\n    <loc>{}</loc>\n    <lastmod>{}</lastmod>\n  </url>",
                escape(&premise_url(site, premise.id)),
                format_timestamp(&premise.updated_at)
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    document("urlset", &urls)
}

/// The published URL of one paged sitemap.
fn page_url(site: &str, page: usize) -> String {
    format!("{site}{PUBLIC_PREFIX}/sitemaps/premises.xml?page={page}")
}

/// The published URL of the page for one premise.
fn premise_url(site: &str, id: i32) -> String {
    format!("{site}/premises?id={id}")
}

/// Wraps sitemap entries in the document header and namespace they belong to.
fn document(root: &str, entries: &str) -> String {
    let body = if entries.is_empty() {
        String::new()
    } else {
        format!("\n{entries}\n")
    };

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<{root} xmlns=\"{NAMESPACE}\">{body}</{root}>\n"
    )
}

/// The five entities XML reserves. A URL is the only thing that ever needs
/// this.
fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(character),
        }
    }

    escaped
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn premise(id: i32, updated_at: &str) -> Premise {
        Premise {
            id,
            updated_at: chrono::DateTime::parse_from_rfc3339(updated_at)
                .expect("timestamp")
                .with_timezone(&Utc),
        }
    }

    #[test]
    fn the_index_lists_a_url_for_every_page() {
        assert_eq!(
            index("https://bins.example.com", 3),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <sitemap>
    <loc>https://bins.example.com/api/sitemaps/premises.xml?page=0</loc>
  </sitemap>
  <sitemap>
    <loc>https://bins.example.com/api/sitemaps/premises.xml?page=1</loc>
  </sitemap>
  <sitemap>
    <loc>https://bins.example.com/api/sitemaps/premises.xml?page=2</loc>
  </sitemap>
</sitemapindex>
"#
        );
    }

    #[test]
    fn the_index_is_empty_when_there_are_no_pages_to_list() {
        assert_eq!(
            index("https://bins.example.com", 0),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"></sitemapindex>
"#
        );
    }

    #[test]
    fn a_page_lists_the_address_url_and_when_it_changed() {
        assert_eq!(
            urlset(
                "https://bins.example.com",
                &[
                    premise(4242, "2026-11-02T09:30:00.123Z"),
                    premise(4243, "2026-11-02T09:30:00.123Z"),
                ]
            ),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://bins.example.com/premises?id=4242</loc>
    <lastmod>2026-11-02T09:30:00.123Z</lastmod>
  </url>
  <url>
    <loc>https://bins.example.com/premises?id=4243</loc>
    <lastmod>2026-11-02T09:30:00.123Z</lastmod>
  </url>
</urlset>
"#
        );
    }

    #[test]
    fn a_page_past_the_last_premise_is_an_empty_urlset() {
        assert_eq!(
            urlset("https://bins.example.com", &[]),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"></urlset>
"#
        );
    }

    #[test]
    fn a_site_with_a_trailing_slash_does_not_produce_a_doubled_one() {
        assert!(
            index("https://bins.example.com/", 1).contains("<loc>https://bins.example.com/api/")
        );
        assert!(
            urlset(
                "https://bins.example.com/",
                &[premise(1, "2026-11-02T09:30:00Z")]
            )
            .contains("<loc>https://bins.example.com/premises?id=1</loc>")
        );
    }

    #[test]
    fn xml_reserved_characters_in_a_url_are_escaped() {
        assert_eq!(
            urlset(
                "https://bins.example.com/?a=1&b=2",
                &[premise(1, "2026-11-02T09:30:00Z")]
            )
            .matches("&amp;b=2")
            .count(),
            1
        );
        assert_eq!(escape("a&b<c>d\"e'f"), "a&amp;b&lt;c&gt;d&quot;e&apos;f");
        // Nothing to escape is left alone.
        assert_eq!(escape("https://x/y?id=1"), "https://x/y?id=1");
    }

    #[test]
    fn a_skip_setting_of_any_kind_is_honoured() {
        assert!(!skipped_by(None));
        assert!(!skipped_by(Some(String::new())));
        assert!(skipped_by(Some("1".to_owned())));
        assert!(skipped_by(Some("false".to_owned())));
    }

    #[tokio::test]
    async fn pages_are_fifty_thousand_of_the_count_rounded_up() {
        // The arithmetic is worth stating on its own, since it is what decides
        // how many sitemaps an index has to list.
        let pages = |count: i64| -> usize {
            count.div_euclid(i64::from(PAGE_SIZE)) as usize
                + usize::from(count.rem_euclid(i64::from(PAGE_SIZE)) > 0)
        };

        assert_eq!(pages(0), 0);
        assert_eq!(pages(1), 1);
        assert_eq!(pages(50_000), 1);
        assert_eq!(pages(50_001), 2);
        assert_eq!(pages(411_448), 9);
    }

    #[test]
    fn timestamps_are_written_as_utc_with_milliseconds() {
        let timestamp = Utc.with_ymd_and_hms(2026, 11, 2, 9, 30, 0).unwrap();

        assert_eq!(format_timestamp(&timestamp), "2026-11-02T09:30:00.000Z");
    }
}
