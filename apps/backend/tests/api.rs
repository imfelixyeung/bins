//! End-to-end checks on the HTTP surface, against a real database.
//!
//! The endpoints read the synced tables, so these need Postgres with PostGIS,
//! which is what the `rewrite-db` compose service provides:
//!
//! ```sh
//! TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5432/db cargo test
//! ```
//!
//! Without `TEST_DATABASE_URL` (or `DATABASE_URL`) they report themselves as
//! skipped rather than failing, so `cargo test` still works on its own.

use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use backend::{db, import::premises::search_postcode, routes, search::format_timestamp, sitemap};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use futures_util::StreamExt;
use reqwest::StatusCode;
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;

struct TestServer {
    base: String,
    /// Held so the tests can insert fixtures, and so the pool outlives the
    /// server task.
    pool: PgPool,
}

impl TestServer {
    async fn start() -> Option<Self> {
        let url = match database_url() {
            Some(url) => url,
            None => {
                eprintln!("skipped: set TEST_DATABASE_URL to run the API tests");
                return None;
            }
        };

        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .unwrap_or_else(|err| panic!("connecting to {url}: {err}"));
        db::migrate(&pool).await.expect("migrations");

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("binding an ephemeral port");
        let address = listener.local_addr().expect("local address");
        let router: Router = routes::router(pool.clone());
        tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        });

        Some(Self {
            base: format!("http://{address}"),
            pool,
        })
    }
    async fn get(&self, path: &str) -> (StatusCode, Value) {
        let response = self.fetch(path).await;

        let body: Value = serde_json::from_str(&response.body).unwrap_or_else(|err| {
            panic!("GET {path} returned non-JSON {:?}: {err}", response.body)
        });

        assert_eq!(
            response.content_type, "application/json",
            "unexpected content type for {path}"
        );
        (response.status, body)
    }

    /// A download, read as text and checked for the headers it is served with,
    /// which name the file.
    async fn get_download(
        &self,
        path: &str,
        content_type: &str,
        filename: &str,
    ) -> (StatusCode, String) {
        let response = self.fetch(path).await;

        assert_eq!(
            response.content_type, content_type,
            "unexpected content type for {path}"
        );
        assert_eq!(
            response.content_disposition.as_deref(),
            Some(format!("attachment; filename=\"{filename}\"").as_str()),
            "unexpected filename for {path}"
        );

        (response.status, response.body)
    }

    /// The sitemaps are XML, so they are read as text and checked for the
    /// headers they are served with.
    async fn get_xml(&self, path: &str) -> (StatusCode, String) {
        let response = self.fetch(path).await;

        assert_eq!(
            response.content_type, "application/xml",
            "unexpected content type for {path}"
        );
        assert_eq!(
            response.cache_control.as_deref(),
            Some("public, max-age=60"),
            "unexpected cache header for {path}"
        );
        (response.status, response.body)
    }

    async fn fetch(&self, path: &str) -> Response {
        let response = reqwest::get(format!("{}{path}", self.base))
            .await
            .unwrap_or_else(|err| panic!("GET {path}: {err}"));

        let status = response.status();
        let header = |name: reqwest::header::HeaderName| {
            response
                .headers()
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned)
        };

        let content_type = header(reqwest::header::CONTENT_TYPE).unwrap_or_default();
        let cache_control = header(reqwest::header::CACHE_CONTROL);
        let content_disposition = header(reqwest::header::CONTENT_DISPOSITION);

        let body = response
            .text()
            .await
            .unwrap_or_else(|err| panic!("reading the body of GET {path}: {err}"));

        Response {
            status,
            content_type,
            cache_control,
            content_disposition,
            body,
        }
    }
}

/// One HTTP response, read once so the helpers above can make several claims
/// about it.
struct Response {
    status: StatusCode,
    content_type: String,
    cache_control: Option<String>,
    content_disposition: Option<String>,
    body: String,
}

/// How many premises sit before `id`, which is what decides the page it lands
/// on: the sitemaps are read in id order, fifty thousand to a page.
async fn page_of(pool: &PgPool, id: i32) -> u32 {
    let (before,): (i64,) = sqlx::query_as("SELECT count(*) FROM dm_premises WHERE id < $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("counting the premises before the fixture");

    (before / 50_000) as u32
}

/// The `<loc>` values in a sitemap or sitemap index, in document order.
fn locations(xml: &str) -> Vec<String> {
    let mut rest = xml;

    std::iter::from_fn(move || {
        let start = rest.find("<loc>")? + "<loc>".len();
        let end = start + rest[start..].find("</loc>")?;
        let value = rest[start..end].to_owned();
        rest = &rest[end..];
        Some(value)
    })
    .collect()
}

fn database_url() -> Option<String> {
    ["TEST_DATABASE_URL", "DATABASE_URL"]
        .into_iter()
        .find_map(|key| std::env::var(key).ok())
}

/// An isolated postcode and id range for one test, so fixtures cannot collide
/// with each other or with synced data. The range is emptied first, because the
/// same database is reused by every run.
struct Slot {
    postcode: String,
    first_id: i32,
    /// Every postcode this test owns, for clearing `postcodes` rows.
    district: String,
}

async fn slot(pool: &PgPool) -> Slot {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let index = NEXT.fetch_add(1, Ordering::SeqCst) + 1;

    let slot = Slot {
        postcode: format!("ZZ{index:02} 1ZZ"),
        first_id: 9_000_000 + index as i32 * 100,
        district: format!("ZZ{index:02} %"),
    };

    sqlx::query("DELETE FROM dm_jobs WHERE premises_id >= $1 AND premises_id < $2")
        .bind(slot.first_id)
        .bind(slot.first_id + 100)
        .execute(pool)
        .await
        .expect("clearing fixture jobs");
    sqlx::query("DELETE FROM dm_premises WHERE search_postcode = $1")
        .bind(search_postcode(&slot.postcode))
        .execute(pool)
        .await
        .expect("clearing fixture premises");
    sqlx::query("DELETE FROM postcodes WHERE id LIKE $1")
        .bind(&slot.district)
        .execute(pool)
        .await
        .expect("clearing fixture postcodes");

    slot
}

/// The second postcode in a slot's district, `ZZ01 2ZZ` for the first test.
fn nearby_postcode(slot: &Slot, index: u8) -> String {
    format!("ZZ{} {}ZZ", &slot.district[2..4], index)
}

/// A postcode with coordinates, which is what `/api/nearby` needs before it can
/// find anything.
async fn postcode(pool: &PgPool, id: &str, latitude: f64, longitude: f64) {
    sqlx::query(
        "INSERT INTO postcodes (id, latitude, longitude) VALUES ($1, $2, $3) \
         ON CONFLICT (id) DO UPDATE SET \
             latitude = EXCLUDED.latitude, longitude = EXCLUDED.longitude",
    )
    .bind(id)
    .bind(latitude)
    .bind(longitude)
    .execute(pool)
    .await
    .expect("inserting a postcode");
}

async fn premise(pool: &PgPool, slot: &Slot, index: i32, address_number: Option<&str>) -> i32 {
    let id = slot.first_id + index;

    sqlx::query(
        "INSERT INTO dm_premises (
             id, address_room, address_number, address_street, address_locality,
             address_city, address_postcode, search_postcode
         ) VALUES ($1, NULL, $2, 'TEST STREET', 'TEST LOCALITY', 'LEEDS', $3, $4)
         ON CONFLICT (id) DO UPDATE SET
             address_number = EXCLUDED.address_number,
             address_postcode = EXCLUDED.address_postcode,
             search_postcode = EXCLUDED.search_postcode",
    )
    .bind(id)
    .bind(address_number)
    .bind(&slot.postcode)
    .bind(search_postcode(&slot.postcode))
    .execute(pool)
    .await
    .expect("inserting a premise");

    id
}

/// A premise at `postcode` rather than at the slot's own postcode, for tests
/// that need collections happening somewhere other than the anchor.
async fn premise_at(pool: &PgPool, slot: &Slot, index: i32, postcode: &str) -> i32 {
    let id = slot.first_id + index;

    sqlx::query(
        "INSERT INTO dm_premises (
             id, address_room, address_number, address_street, address_locality,
             address_city, address_postcode, search_postcode
         ) VALUES ($1, NULL, NULL, 'TEST STREET', 'TEST LOCALITY', 'LEEDS', $2, $3)
         ON CONFLICT (id) DO UPDATE SET
             address_postcode = EXCLUDED.address_postcode,
             search_postcode = EXCLUDED.search_postcode",
    )
    .bind(id)
    .bind(postcode)
    .bind(search_postcode(postcode))
    .execute(pool)
    .await
    .expect("inserting a premise");

    id
}

async fn jobs(pool: &PgPool, premises_id: i32, collections: &[(&str, &str)]) {
    // Cleared first so repeated runs against the same database do not pile up.
    sqlx::query("DELETE FROM dm_jobs WHERE premises_id = $1")
        .bind(premises_id)
        .execute(pool)
        .await
        .expect("clearing jobs");

    for (bin, date) in collections {
        sqlx::query("INSERT INTO dm_jobs (premises_id, bin, date) VALUES ($1, $2, $3)")
            .bind(premises_id)
            .bind(bin)
            .bind(NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a fixture date"))
            .execute(pool)
            .await
            .expect("inserting a job");
    }
}

#[tokio::test]
async fn premises_returns_every_address_at_the_postcode() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;

    // Inserted out of order, and with numbers that only sort correctly when read
    // as numbers rather than text.
    premise(&server.pool, &slot, 0, Some("10")).await;
    premise(&server.pool, &slot, 1, Some("9")).await;
    let unnumbered = premise(&server.pool, &slot, 2, None).await;
    premise(&server.pool, &slot, 3, Some("100")).await;

    // Lower case and unspaced, as the web app sends it.
    let (status, body) = server
        .get(&format!(
            "/api/premises?postcode={}",
            slot.postcode.replace(' ', "").to_lowercase()
        ))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["timestamp"].as_str().is_some_and(|t| t.ends_with('Z')));

    let data = body["data"].as_array().expect("an array of premises");
    let numbers: Vec<Value> = data.iter().map(|p| p["addressNumber"].clone()).collect();
    assert_eq!(
        numbers,
        vec![json!(null), json!("9"), json!("10"), json!("100")],
        "house numbers should sort numerically, unnumbered addresses first"
    );

    assert_eq!(data[1]["id"], slot.first_id + 1);
    assert_eq!(data[1]["addressRoom"], Value::Null);
    assert_eq!(data[1]["addressStreet"], "TEST STREET");
    assert_eq!(data[1]["addressLocality"], "TEST LOCALITY");
    assert_eq!(data[1]["addressCity"], "LEEDS");
    assert_eq!(data[1]["addressPostcode"], slot.postcode);
    assert_eq!(
        data[1].as_object().expect("an object").len(),
        8,
        "only the documented address fields should be served"
    );
    assert_ne!(data[1]["updatedAt"], Value::Null);

    // The premise the id came from is the one with no house number.
    assert_eq!(data[0]["id"], unnumbered);
}

#[tokio::test]
async fn random_premises_returns_a_stored_premise() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    let (status, body) = server.get("/api/random/premises").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["timestamp"].as_str().is_some_and(|t| t.ends_with('Z')));

    let data = &body["data"];
    let id = data["id"].as_i64().expect("a premise id");
    // Most premises carry a postcode, but the feed has rows without one, and a
    // random pick lands on one of those every so often.
    match &data["addressPostcode"] {
        Value::String(postcode) => assert!(!postcode.is_empty(), "a blank postcode"),
        Value::Null => {}
        other => panic!("unexpected postcode {other}"),
    }
    // The whole address, matching the `/api/premises` payload.
    assert_eq!(
        data.as_object().expect("an address").len(),
        8,
        "the payload should match the one from /api/premises"
    );

    // And it is a row that exists, rather than an id made up on the way out.
    let stored: Option<(i32, String)> =
        sqlx::query_as("SELECT id, search_postcode FROM dm_premises WHERE id = $1")
            .bind(i32::try_from(id).expect("an id Postgres holds"))
            .fetch_optional(&server.pool)
            .await
            .expect("querying the premise that came back");
    assert_eq!(stored.map(|(stored, _)| stored), Some(id as i32));
}

#[tokio::test]
async fn random_premises_takes_no_query_string() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    // Unknown parameters are ignored, as they are everywhere else, so a
    // postcode sent by mistake cannot narrow or empty the result.
    let (status, body) = server
        .get("/api/random/premises?postcode=ZZ99%209ZZ&format=csv")
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["id"].as_i64().is_some());
}

#[tokio::test]
async fn nearby_returns_the_closest_postcodes_with_their_collections() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;

    // Mid Atlantic, where `sync postcodes` has nothing to write, so the only
    // postcodes within reach are this test's own. One degree of latitude is
    // about 111km, so the second is a kilometre away and the third is well
    // outside the two kilometre radius.
    let neighbour = nearby_postcode(&slot, 2);
    let faraway = nearby_postcode(&slot, 3);
    postcode(&server.pool, &slot.postcode, 40.0, -40.0).await;
    postcode(&server.pool, &neighbour, 40.01, -40.0).await;
    postcode(&server.pool, &faraway, 40.10, -40.0).await;

    // Both nearby postcodes collect, and two addresses at the anchor postcode
    // share one collection, which `/api/nearby` should collapse into one row.
    let id = premise(&server.pool, &slot, 0, Some("6")).await;
    jobs(&server.pool, id, &[("BLACK", "2026-11-02")]).await;
    let second = premise(&server.pool, &slot, 1, Some("8")).await;
    jobs(
        &server.pool,
        second,
        &[("BLACK", "2026-11-02"), ("GREEN", "2026-11-02")],
    )
    .await;

    let neighbour_id = premise_at(&server.pool, &slot, 2, &neighbour).await;
    jobs(&server.pool, neighbour_id, &[("GREEN", "2026-11-09")]).await;
    let faraway_id = premise_at(&server.pool, &slot, 3, &faraway).await;
    jobs(&server.pool, faraway_id, &[("BLACK", "2026-11-16")]).await;

    // Lower case and unspaced, as the web app sends it.
    let (status, body) = server
        .get(&format!(
            "/api/nearby?postcode={}",
            slot.postcode.replace(' ', "").to_lowercase()
        ))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);

    let data = body["data"].as_array().expect("a list of nearby postcodes");
    assert_eq!(data.len(), 2, "the postcode 11km away is out of range");

    // Nearest first, and the postcode asked about is always at zero metres.
    assert_eq!(data[0]["postcode"], slot.postcode);
    assert_eq!(data[0]["distance"], 0.0);
    assert_eq!(data[0]["latitude"], 40.0);
    assert_eq!(data[0]["longitude"], -40.0);
    assert_eq!(data[1]["postcode"], neighbour);
    assert!(
        data[1]["distance"]
            .as_f64()
            .is_some_and(|metres| (1000.0..1200.0).contains(&metres)),
        "expected about a kilometre, got {:?}",
        data[1]["distance"]
    );

    // Collections are per postcode, bin and date, with duplicates collapsed.
    assert_eq!(
        data[0]["jobs"],
        json!([
            { "bin": "BLACK", "date": "2026-11-02", "postcode": slot.postcode },
            { "bin": "GREEN", "date": "2026-11-02", "postcode": slot.postcode },
        ])
    );
    assert_eq!(
        data[1]["jobs"],
        json!([{ "bin": "GREEN", "date": "2026-11-09", "postcode": neighbour }])
    );
}

#[tokio::test]
async fn nearby_is_null_for_a_postcode_it_has_no_coordinates_for() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;

    // The premises are there, but `sync postcodes` has not reached the postcode,
    // so there is nothing to search outwards from.
    premise(&server.pool, &slot, 0, Some("6")).await;

    let (status, body) = server.get("/api/nearby?postcode=ZZ99%209ZZ").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"], Value::Null);
}

#[tokio::test]
async fn nearby_needs_a_postcode() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    let (status, body) = server.get("/api/nearby").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        body,
        json!({
            "error": true,
            "issues": [{
                "code": "invalid_type",
                "expected": "string",
                "received": "undefined",
                "path": ["postcode"],
                "message": "Required",
            }]
        })
    );
}

#[tokio::test]
async fn premises_answers_with_an_empty_list_for_an_unknown_postcode() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    let (status, body) = server.get("/api/premises?postcode=ZZ99 9ZZ").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"], json!([]));
}

#[tokio::test]
async fn premises_needs_a_postcode() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    let (status, body) = server.get("/api/premises").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], true);
    assert_eq!(body["issues"][0]["path"], json!(["postcode"]));
}

#[tokio::test]
async fn jobs_returns_the_premise_with_its_collections() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;
    let id = premise(&server.pool, &slot, 0, Some("6")).await;

    jobs(
        &server.pool,
        id,
        &[
            ("GREEN", "2026-11-02"),
            ("BLACK", "2026-11-02"),
            ("BROWN", "2026-07-26"),
        ],
    )
    .await;

    let (status, body) = server.get(&format!("/api/jobs?premises={id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);

    let data = &body["data"];
    assert_eq!(data["id"], id);
    assert_eq!(data["addressNumber"], "6");
    assert_eq!(data["addressStreet"], "TEST STREET");
    assert_eq!(data["addressPostcode"], slot.postcode);
    assert!(data["updatedAt"].as_str().is_some_and(|t| t.ends_with('Z')));

    assert_eq!(
        data["jobs"],
        json!([
            { "bin": "BROWN", "date": "2026-07-26" },
            { "bin": "BLACK", "date": "2026-11-02" },
            { "bin": "GREEN", "date": "2026-11-02" },
        ]),
        "jobs sort by date, then bin"
    );
}

#[tokio::test]
async fn jobs_returns_a_premise_that_has_no_collections() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;
    let id = premise(&server.pool, &slot, 0, Some("6")).await;
    jobs(&server.pool, id, &[]).await;

    let (status, body) = server.get(&format!("/api/jobs?premises={id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["id"], id);
    assert_eq!(body["data"]["jobs"], json!([]));
}

#[tokio::test]
async fn jobs_is_not_found_for_an_unknown_premise() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    let (status, body) = server.get("/api/jobs?premises=1").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        body,
        json!({ "error": true, "message": "Premises not found" })
    );
}

#[tokio::test]
async fn jobs_needs_a_premises_id() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    for query in ["/api/jobs", "/api/jobs?premises=", "/api/jobs?premises=abc"] {
        let (status, body) = server.get(query).await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}");
        assert_eq!(body["error"], true, "{query}");
        assert_eq!(body["issues"][0]["path"], json!(["premises"]), "{query}");
    }
}

#[tokio::test]
async fn jobs_ignores_unknown_query_parameters() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;
    let id = premise(&server.pool, &slot, 0, Some("6")).await;

    let (status, body) = server
        .get(&format!("/api/jobs?premises={id}&format=json&locale=en-GB"))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["id"], id);
}

#[tokio::test]
async fn jobs_reports_every_problem_at_once() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    let (status, body) = server.get("/api/jobs?premises=abc&format=xml").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        body["issues"],
        json!([
            {
                "code": "invalid_type",
                "expected": "number",
                "received": "nan",
                "path": ["premises"],
                "message": "Expected number, received nan",
            },
            {
                "code": "invalid_enum_value",
                "options": ["json", "csv", "ical"],
                "received": "xml",
                "path": ["format"],
                "message": "Invalid enum value. Expected 'json' | 'csv' | 'ical', received 'xml'",
            },
        ])
    );
}

/// A calendar with its folded lines joined back up, which is how a calendar app
/// reads one.
fn unfold(body: &str) -> String {
    body.replace("\r\n ", "")
}

#[tokio::test]
async fn jobs_serves_a_csv_of_the_collections() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;
    let id = premise(&server.pool, &slot, 0, Some("6")).await;
    jobs(
        &server.pool,
        id,
        &[("GREEN", "2026-11-02"), ("BLACK", "2026-11-09")],
    )
    .await;

    let (status, body) = server
        .get_download(
            &format!("/api/jobs?premises={id}&format=csv"),
            "text/csv",
            &format!("jobs-{id}.csv"),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "Date,Bin\n2026-11-02,GREEN\n2026-11-09,BLACK\n");
}

#[tokio::test]
async fn jobs_serves_a_calendar_of_the_collections() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;
    let id = premise(&server.pool, &slot, 0, Some("6")).await;
    let postcode = slot.postcode.as_str();
    jobs(
        &server.pool,
        id,
        &[("GREEN", "2026-11-02"), ("BLACK", "2026-11-09")],
    )
    .await;

    let (status, body) = server
        .get_download(
            &format!("/api/jobs?premises={id}&format=ical"),
            "text/calendar",
            &format!("jobs-{id}.ics"),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.starts_with("BEGIN:VCALENDAR\r\n"), "{body}");
    assert!(body.ends_with("END:VCALENDAR\r\n"), "{body}");

    let calendar = unfold(&body);

    // One event per collection, in date order.
    assert_eq!(calendar.matches("BEGIN:VEVENT").count(), 2, "{calendar}");
    assert!(
        calendar.contains(&format!("UID:{id}_2026-11-02_GREEN")),
        "{calendar}"
    );
    assert!(
        calendar.contains(&format!("UID:{id}_2026-11-09_BLACK")),
        "{calendar}"
    );

    // Named after the household, described with where the bins are.
    assert!(
        calendar.contains("X-WR-CALNAME:Bin Days for 6 TEST STREET"),
        "{calendar}"
    );
    assert!(
        calendar.contains(&format!("X-WR-CALDESC:Bin collection dates for your household\\n\\n6\\nTEST STREET\\nTEST LOCALITY\\nLEEDS\\n{postcode}\\n\\nID: {id}")),
        "{calendar}"
    );

    // Every event is a whole day, collected from the address, and named after
    // the bin.
    assert_eq!(
        calendar.matches("DTSTART;VALUE=DATE:20261102").count(),
        1,
        "{calendar}"
    );
    assert_eq!(
        calendar.matches("DTSTART;VALUE=DATE:20261109").count(),
        1,
        "{calendar}"
    );
    assert!(
        calendar.contains("SUMMARY:Green Bin Collection"),
        "{calendar}"
    );
    assert!(
        calendar.contains(&format!(
            "LOCATION:6\\nTEST STREET\\nTEST LOCALITY\\nLEEDS\\n{postcode}"
        )),
        "{calendar}"
    );
    assert!(
        calendar.contains(&format!(
            "URL;VALUE=URI:https://bins.felixyeung.com/premises?id={id}&date=2026-11-02&bin=GREEN"
        )),
        "{calendar}"
    );
}

#[tokio::test]
async fn the_premises_sitemap_index_lists_a_page_for_every_fifty_thousand_premises() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    // A premise to make sure the table is not empty.
    let slot = slot(&server.pool).await;
    premise(&server.pool, &slot, 0, Some("6")).await;

    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM dm_premises")
        .fetch_one(&server.pool)
        .await
        .expect("counting the premises");
    let expected_pages = (count / 50_000 + i64::from(count % 50_000 > 0)) as usize;
    assert!(expected_pages > 0, "there should be premises to list");

    let (status, body) = server.get_xml("/api/sitemaps/premises.xml").await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<sitemapindex"),
        "not a sitemap index: {body}"
    );
    assert!(
        body.contains("xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\""),
        "{body}"
    );

    // One URL per page, from the first to the last, under the public prefix the
    // proxy serves rather than the path this server answers on.
    let site = sitemap::site();
    let urls = locations(&body);
    assert_eq!(urls.len(), expected_pages, "{body}");
    assert_eq!(
        urls.first().unwrap(),
        &format!("{site}/api/sitemaps/premises.xml?page=0")
    );
    assert_eq!(
        urls.last().unwrap(),
        &format!(
            "{site}/api/sitemaps/premises.xml?page={}",
            expected_pages - 1
        )
    );
    assert!(
        urls.iter()
            .all(|url| url.starts_with(&format!("{site}/api/"))),
        "{urls:?}"
    );
}

#[tokio::test]
async fn a_premises_sitemap_page_lists_the_addresses_read_onto_it() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;
    let first = premise(&server.pool, &slot, 0, Some("6")).await;
    let second = premise(&server.pool, &slot, 1, Some("8")).await;

    let page = page_of(&server.pool, first).await;
    let (status, body) = server
        .get_xml(&format!("/api/sitemaps/premises.xml?page={page}"))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset"),
        "not a urlset: {body}"
    );

    // The fixtures are on the page their id count puts them on, oldest first,
    // each pointing at the address page with when that address changed.
    let site = sitemap::site();
    let urls = locations(&body);
    let ids: Vec<i32> = urls
        .iter()
        .map(|url| {
            url.strip_prefix(&format!("{site}/premises?id="))
                .unwrap_or_else(|| panic!("unexpected url {url}"))
                .parse()
                .expect("a premise id in a sitemap url")
        })
        .collect();

    assert!(
        ids.contains(&first) && ids.contains(&second),
        "the fixtures are missing from {urls:?}"
    );
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "premises should be listed in id order");
    assert!(!ids.is_empty(), "the page should hold the fixtures");
    assert!(
        ids.len() <= 50_000,
        "a page never holds more than fifty thousand"
    );

    let (_, updated_at): (i32, DateTime<Utc>) =
        sqlx::query_as("SELECT id, updated_at FROM dm_premises WHERE id = $1")
            .bind(first)
            .fetch_one(&server.pool)
            .await
            .expect("reading the fixture");

    assert!(
        body.contains(&format!(
            "  <url>\n    <loc>{site}/premises?id={first}</loc>\n    \
             <lastmod>{}</lastmod>\n  </url>",
            format_timestamp(&updated_at)
        )),
        "{body}"
    );
}

#[tokio::test]
async fn a_premises_sitemap_page_past_the_last_one_holds_no_addresses() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    // An empty sitemap is what the query returns, and is easier for a crawler to
    // move past than a 404.
    let (status, body) = server.get_xml("/api/sitemaps/premises.xml?page=9999").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(locations(&body), Vec::<String>::new());
    assert!(body.contains("<urlset"), "{body}");
}

#[tokio::test]
async fn a_premises_sitemap_page_has_to_be_a_number() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    for page in ["abc", "-1", "1.5", ""] {
        let (status, body) = server
            .get(&format!(
                "/api/sitemaps/premises.xml?page={}",
                urlencode(page)
            ))
            .await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "page={page:?}");
        assert_eq!(
            body,
            json!({
                "error": true,
                "issues": [{
                    "code": "invalid_type",
                    "expected": "number",
                    "received": "nan",
                    "path": ["page"],
                    "message": "Expected number, received nan",
                }]
            }),
            "page={page:?}"
        );
    }
}

/// Percent-encodes a query value the way a caller would send it.
fn urlencode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// The datasets `/api/datasets` reports on, which are the two the sync commands
/// default to.
const DATASETS: [(&str, &str, &str); 2] = [
    (
        "jobs",
        "Jobs",
        "https://opendata.leeds.gov.uk/downloads/bins/dm_jobs.csv",
    ),
    (
        "premises",
        "Premises",
        "https://opendata.leeds.gov.uk/downloads/bins/dm_premises.csv",
    ),
];

/// The lock every dataset test holds while it is reading the datasets above.
///
/// Unlike the fixtures elsewhere in this file, a dataset test cannot be given a
/// dataset of its own: `/api/datasets` reports on the datasets the sync commands
/// default to, so these are the rows every dataset test reads. Tests run
/// concurrently, so this is what keeps them from clearing each other out
/// mid-request.
fn datasets_lock() -> std::sync::Arc<tokio::sync::Mutex<()>> {
    static LOCK: std::sync::OnceLock<std::sync::Arc<tokio::sync::Mutex<()>>> =
        std::sync::OnceLock::new();

    LOCK.get_or_init(|| std::sync::Arc::new(tokio::sync::Mutex::new(())))
        .clone()
}

/// An `etags` row as a test found it.
#[derive(sqlx::FromRow)]
struct Etag {
    id: i32,
    url: String,
    etag: Option<String>,
    modified_at: Option<DateTime<Utc>>,
    checked_at: DateTime<Utc>,
    size: Option<i64>,
}

/// A `sync_runs` row as a test found it.
#[derive(sqlx::FromRow)]
struct SyncRun {
    id: i32,
    target: String,
    source: String,
    state: String,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    rows: Option<i64>,
    message: Option<String>,
}

/// The datasets above, as a test finds them and as it leaves them.
///
/// A dataset test has to write over the rows a real sync left, because the
/// datasets `/api/datasets` reports on are the ones the sync commands default
/// to. They are read out first and written back by [`DatasetFixture::restore`],
/// since the alternative is a test run deleting a real sync's record of the
/// file the database holds.
struct DatasetFixture {
    /// Held for the length of the test, so no other dataset test replaces these
    /// rows while the first test is still reading them.
    _lock: tokio::sync::OwnedMutexGuard<()>,
    pool: PgPool,
    etags: Vec<Etag>,
    runs: Vec<SyncRun>,
}

impl DatasetFixture {
    /// Puts the datasets back to how this test found it.
    async fn restore(self) {
        clear_datasets(&self.pool).await;

        for etag in &self.etags {
            sqlx::query(
                "INSERT INTO etags (id, url, etag, modified_at, checked_at, size) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(etag.id)
            .bind(&etag.url)
            .bind(&etag.etag)
            .bind(etag.modified_at)
            .bind(etag.checked_at)
            .bind(etag.size)
            .execute(&self.pool)
            .await
            .expect("restoring an etag a dataset test replaced");
        }

        for run in &self.runs {
            sqlx::query(
                "INSERT INTO sync_runs (
                     id, target, source, state, started_at, finished_at, rows, message
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(run.id)
            .bind(&run.target)
            .bind(&run.source)
            .bind(&run.state)
            .bind(run.started_at)
            .bind(run.finished_at)
            .bind(run.rows)
            .bind(&run.message)
            .execute(&self.pool)
            .await
            .expect("restoring a sync run a dataset test replaced");
        }
    }
}

/// Reads the datasets above, clears them for a test to fill in, and holds the
/// lock until [`DatasetFixture::restore`] hands it back.
async fn dataset_fixture(pool: &PgPool) -> DatasetFixture {
    let lock = datasets_lock().lock_owned().await;

    let mut etags = Vec::new();
    let mut runs = Vec::new();
    for (target, _, url) in DATASETS {
        etags.extend(
            sqlx::query_as::<_, Etag>(
                "SELECT id, url, etag, modified_at, checked_at, size FROM etags WHERE url = $1",
            )
            .bind(url)
            .fetch_all(pool)
            .await
            .expect("reading the etags a dataset test replaces"),
        );
        runs.extend(
            sqlx::query_as::<_, SyncRun>(
                "SELECT id, target, source, state, started_at, finished_at, rows, message \
                 FROM sync_runs WHERE target = $1",
            )
            .bind(target)
            .fetch_all(pool)
            .await
            .expect("reading the sync runs a dataset test replaces"),
        );
    }

    clear_datasets(pool).await;

    DatasetFixture {
        _lock: lock,
        pool: pool.clone(),
        etags,
        runs,
    }
}

/// Empties the datasets above.
async fn clear_datasets(pool: &PgPool) {
    for (target, _, url) in DATASETS {
        sqlx::query("DELETE FROM sync_runs WHERE target = $1")
            .bind(target)
            .execute(pool)
            .await
            .expect("clearing fixture sync runs");
        sqlx::query("DELETE FROM etags WHERE url = $1")
            .bind(url)
            .execute(pool)
            .await
            .expect("clearing fixture etags");
    }
}

/// How big the datasets' mirrors are, which depends on which fixtures other
/// tests have synced into the same database. A null size means the table is
/// empty, and anything else is a size in bytes.
fn assert_size_type(value: &Value) {
    match value.as_i64() {
        Some(size) => assert!(size > 0, "a table size is positive bytes"),
        None => assert_eq!(value, &Value::Null, "a table size is null or positive"),
    }
}

#[tokio::test]
async fn datasets_report_what_the_dataset_page_reads() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let fixtures = dataset_fixture(&server.pool).await;

    // What a sync leaves behind after importing a file that had changed.
    sqlx::query(
        "INSERT INTO etags (url, etag, modified_at, checked_at, size) \
         VALUES ($1, '\"f01141fe7b54dd1:0\"', $2, $3, $4)",
    )
    .bind(DATASETS[1].2)
    .bind(Utc.with_ymd_and_hms(2026, 10, 4, 3, 45, 41).unwrap())
    .bind(Utc.with_ymd_and_hms(2026, 10, 5, 4, 0, 0).unwrap())
    .bind(170_000_000_i64)
    .execute(&server.pool)
    .await
    .expect("inserting an etag");
    sqlx::query(
        "INSERT INTO sync_runs (target, source, state, started_at, finished_at, rows) \
         VALUES ($1, $2, 'synced', $3, $4, $5)",
    )
    .bind("premises")
    .bind(DATASETS[1].2)
    .bind(Utc.with_ymd_and_hms(2026, 10, 5, 4, 0, 30).unwrap())
    .bind(Utc.with_ymd_and_hms(2026, 10, 5, 4, 1, 2).unwrap())
    .bind(411_468_i64)
    .execute(&server.pool)
    .await
    .expect("inserting a sync run");

    let (status, body) = server.get("/api/datasets").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);

    // Both datasets are listed whether or not anything is known about them, so
    // the page does not have to know which ones have been synced.
    let datasets = body["data"]["datasets"].as_array().expect("the datasets");
    assert_eq!(datasets.len(), DATASETS.len());

    for (dataset, (key, name, url)) in datasets.iter().zip(DATASETS) {
        assert_eq!(dataset["key"], json!(key));
        assert_eq!(dataset["name"], json!(name));
        assert_eq!(dataset["url"], json!(url));
    }

    let synced = &datasets[1];
    assert_eq!(synced["etag"], json!("\"f01141fe7b54dd1:0\""));
    assert_eq!(synced["lastChecked"], json!("2026-10-05T04:00:00.000Z"));
    // The page called this "last updated": it is when the imported file changed
    // upstream, not when the import ran.
    assert_eq!(synced["lastSynced"], json!("2026-10-04T03:45:41.000Z"));
    assert_eq!(synced["csvSize"], json!(170000000));
    assert_size_type(&synced["dbSize"]);
    assert!(synced["sync"]["id"].as_i64().is_some());
    assert_eq!(synced["sync"]["source"], json!(DATASETS[1].2));
    assert_eq!(synced["sync"]["state"], json!("synced"));
    assert_eq!(
        synced["sync"]["startedAt"],
        json!("2026-10-05T04:00:30.000Z")
    );
    assert_eq!(
        synced["sync"]["finishedAt"],
        json!("2026-10-05T04:01:02.000Z")
    );
    assert_eq!(synced["sync"]["rows"], json!(411468));
    assert_eq!(synced["sync"]["message"], Value::Null);

    // The jobs dataset has not been synced here, so its fields are null rather
    // than missing from the response.
    let unsynced = &datasets[0];
    assert_eq!(unsynced["etag"], Value::Null);
    assert_eq!(unsynced["lastChecked"], Value::Null);
    assert_eq!(unsynced["lastSynced"], Value::Null);
    assert_eq!(unsynced["csvSize"], Value::Null);
    assert_size_type(&unsynced["dbSize"]);
    assert_eq!(unsynced["sync"], Value::Null);

    fixtures.restore().await;
}

#[tokio::test]
async fn datasets_report_a_sync_that_is_still_going() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let fixtures = dataset_fixture(&server.pool).await;

    // Started but not finished, which is what a long import of the jobs file
    // looks like while it is happening.
    sqlx::query("INSERT INTO sync_runs (target, source, state) VALUES ($1, $2, 'running')")
        .bind("jobs")
        .bind(DATASETS[0].2)
        .execute(&server.pool)
        .await
        .expect("inserting a sync run");

    let (status, body) = server.get("/api/datasets").await;

    assert_eq!(status, StatusCode::OK);

    let jobs = &body["data"]["datasets"][0];
    assert_eq!(jobs["sync"]["state"], json!("running"));
    assert_eq!(jobs["sync"]["finishedAt"], Value::Null);
    assert_eq!(jobs["sync"]["rows"], Value::Null);
    assert_eq!(jobs["sync"]["message"], Value::Null);

    fixtures.restore().await;
}

#[tokio::test]
async fn datasets_report_a_sync_that_failed() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let fixtures = dataset_fixture(&server.pool).await;

    // A run that failed before any etag was stored still has to say so, since
    // the page is where a broken sync is looked for.
    sqlx::query(
        "INSERT INTO sync_runs (target, source, state, finished_at, message) \
         VALUES ($1, $2, 'failed', now(), $3)",
    )
    .bind("premises")
    .bind(DATASETS[1].2)
    .bind("502 Bad Gateway")
    .execute(&server.pool)
    .await
    .expect("inserting a sync run");

    let (status, body) = server.get("/api/datasets").await;

    assert_eq!(status, StatusCode::OK);

    let premises = &body["data"]["datasets"][1];
    assert_eq!(premises["sync"]["state"], json!("failed"));
    assert_eq!(premises["sync"]["message"], json!("502 Bad Gateway"));
    assert_eq!(premises["etag"], Value::Null);

    fixtures.restore().await;
}

#[tokio::test]
async fn a_dataset_lists_every_sync_run_newest_first() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let fixtures = dataset_fixture(&server.pool).await;

    // Three runs of the premises dataset, in reverse chronological order, which
    // is how the history page reads them back.
    for (state, seconds, rows) in [
        ("synced", 12, 411_468_i64),
        ("unchanged", 13, 0_i64),
        ("failed", 14, 0_i64),
    ] {
        sqlx::query(
            "INSERT INTO sync_runs (target, source, state, started_at, finished_at, rows, message) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind("premises")
        .bind(DATASETS[1].2)
        .bind(state)
        .bind(Utc.with_ymd_and_hms(2026, 10, 5, 4, 0, seconds).unwrap())
        .bind(Some(
            Utc.with_ymd_and_hms(2026, 10, 5, 4, 1, seconds).unwrap(),
        ))
        .bind(rows)
        .bind(Option::<&str>::None)
        .execute(&server.pool)
        .await
        .expect("inserting a sync run");
    }

    let (status, body) = server.get("/api/datasets/premises").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);

    let runs = body["data"].as_array().expect("the run history");
    assert_eq!(runs.len(), 3);
    assert_eq!(
        runs.iter()
            .map(|run| run["state"].as_str().expect("a state"))
            .collect::<Vec<_>>(),
        vec!["failed", "unchanged", "synced"],
        "history reads newest first"
    );
    assert_eq!(runs[2]["rows"], json!(411468));
    assert_eq!(runs[2]["startedAt"], json!("2026-10-05T04:00:12.000Z"));

    fixtures.restore().await;
}

#[tokio::test]
async fn the_history_of_a_dataset_this_build_does_not_hold_is_not_found() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    let (status, body) = server.get("/api/datasets/postcodes").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        body,
        json!({ "error": true, "message": "Dataset not found" })
    );
}

/// The protocol revision these tests speak, which is a revision that keeps a
/// session open.
const PROTOCOL_VERSION: &str = "2025-06-18";

/// A client for `/api/mcp`, speaking the protocol over HTTP as a real one does:
/// one handshake, then every request carrying the session the handshake handed
/// back.
struct Mcp<'a> {
    base: &'a str,
    client: reqwest::Client,
    session: String,
    /// The protocol version the server settled on, which it sends back on every
    /// request from here on.
    version: String,
}

impl TestServer {
    /// The handshake, which is what a client does before anything else.
    async fn mcp(&self) -> Mcp<'_> {
        let client = reqwest::Client::new();

        let response = client
            .post(format!("{}/api/mcp", self.base))
            .header(
                reqwest::header::ACCEPT,
                "application/json, text/event-stream",
            )
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 0,
                "method": "initialize",
                "params": {
                    "protocolVersion": PROTOCOL_VERSION,
                    "capabilities": {},
                    "clientInfo": { "name": "api-tests", "version": "0.0.0" },
                }
            }))
            .send()
            .await
            .expect("the MCP handshake");

        assert_eq!(response.status(), StatusCode::OK);

        let session = response
            .headers()
            .get("mcp-session-id")
            .and_then(|value| value.to_str().ok())
            .expect("a session the handshake opens")
            .to_owned();

        let message = event(response).await;
        let result = &message["result"];

        assert_eq!(result["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(result["serverInfo"]["name"], "bins");

        // The client says it is ready before it asks for anything, as the spec
        // asks it to.
        client
            .post(format!("{}/api/mcp", self.base))
            .header(
                reqwest::header::ACCEPT,
                "application/json, text/event-stream",
            )
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header("mcp-session-id", &session)
            .header("mcp-protocol-version", PROTOCOL_VERSION)
            .json(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }))
            .send()
            .await
            .expect("saying the client is ready");

        Mcp {
            base: &self.base,
            client,
            session,
            version: PROTOCOL_VERSION.to_string(),
        }
    }
}

impl Mcp<'_> {
    /// A request sent the way a client sends every one of them, left as the
    /// response itself so a caller can read an answer that is not a result.
    async fn post(&self, message: Value) -> reqwest::Response {
        self.client
            .post(format!("{}/api/mcp", self.base))
            .header(
                reqwest::header::ACCEPT,
                "application/json, text/event-stream",
            )
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header("mcp-session-id", &self.session)
            .header("mcp-protocol-version", &self.version)
            .json(&message)
            .send()
            .await
            .expect("a request made in the session")
    }

    /// One request, answered with the `result` it was looking for. A failure
    /// answer is left in place for the caller to look at, rather than panicking
    /// here.
    async fn request(&self, id: i64, method: &str, params: Value) -> Value {
        let response = self
            .post(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))
            .await;

        assert_eq!(response.status(), StatusCode::OK, "{method}");

        let message = event(response).await;

        message
            .get("result")
            .cloned()
            .unwrap_or_else(|| panic!("{method} answered with {message}"))
    }

    /// The tools the server offers, in the order it lists them.
    async fn tools(&self) -> Vec<Value> {
        let result = self.request(1, "tools/list", json!({})).await;

        result["tools"]
            .as_array()
            .expect("an array of tools")
            .clone()
    }

    /// One tool, called as a client would call it.
    async fn call(&self, name: &str, arguments: Value) -> Value {
        self.request(
            2,
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        )
        .await
    }
}

/// The answer to one request, which comes back as a server sent event while the
/// session is open. The event is read as it arrives rather than by waiting for
/// the stream to end, since the stream stays open for the rest of the session.
async fn event(response: reqwest::Response) -> Value {
    let mut events = response.bytes_stream();
    let mut read = String::new();

    while let Some(chunk) = events.next().await {
        let chunk = chunk.expect("a chunk of the event stream");
        read.push_str(std::str::from_utf8(&chunk).expect("an event stream of text"));

        // Only the lines that have arrived whole, since a line may still be on
        // its way in the next chunk.
        let Some(last_line) = read.rfind('\n') else {
            continue;
        };
        let arrived = read[..=last_line].to_owned();
        read = read[last_line + 1..].to_owned();

        for line in arrived.lines() {
            let Some(payload) = line.strip_prefix("data:") else {
                continue;
            };

            let payload = payload.trim();

            // The stream opens with a priming event carrying a retry hint and no
            // message, which a client reads past to reach the answer.
            if payload.is_empty() {
                continue;
            }

            return serde_json::from_str(payload)
                .unwrap_or_else(|err| panic!("an answer that is not JSON ({payload:?}): {err}"));
        }
    }

    panic!("the event stream ended without an answer")
}

/// The text blocks of an answer, which is the part of it a person reads.
fn text_of(result: &Value) -> Vec<String> {
    result["content"]
        .as_array()
        .expect("an array of content blocks")
        .iter()
        .map(|block| {
            assert_eq!(
                block["type"], "text",
                "every block the tools answer with is text"
            );

            block["text"]
                .as_str()
                .expect("a text block of text")
                .to_owned()
        })
        .collect()
}

#[tokio::test]
async fn the_handshake_says_what_the_server_is_and_opens_a_session() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    let mcp = server.mcp().await;

    assert!(
        !mcp.session.is_empty(),
        "the session every request is made in"
    );

    let tools = mcp.tools().await;

    assert!(
        tools
            .iter()
            .any(|tool| tool["name"] == "search_premises_from_postcode"),
        "the handshake answered on a session that serves the tools: {tools:?}"
    );
}

#[tokio::test]
async fn the_tools_are_the_three_the_next_js_route_registered() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    let mut names: Vec<String> = server
        .mcp()
        .await
        .tools()
        .await
        .iter()
        .filter_map(|tool| tool["name"].as_str().map(str::to_owned))
        .collect();
    names.sort();

    assert_eq!(
        names,
        vec![
            "get_premises_permalink".to_string(),
            "search_premises_from_postcode".to_string(),
            "show_premises_jobs_by_id".to_string(),
        ],
        "the tools are named as they always have been, listed by name"
    );
}

#[tokio::test]
async fn a_tool_says_what_it_wants_and_what_it_answers_with() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    let tools = server.mcp().await.tools().await;

    let search = tools
        .iter()
        .find(|tool| tool["name"] == "search_premises_from_postcode")
        .expect("the postcode search");

    assert_eq!(
        search["description"],
        "Search premises or addresses using a postcode. The premises.id can then be used in \
         show_premises_jobs_by_id or get_premises_permalink for their respective functions"
    );
    assert_eq!(
        search["inputSchema"]["required"],
        json!(["postcode"]),
        "the postcode is the only thing it needs"
    );
    assert!(
        search["inputSchema"]["properties"]["postcode"]["description"]
            .as_str()
            .is_some_and(|description| description.contains("LS6 2SE")),
        "the parameter carries an example"
    );
    assert_eq!(
        search["outputSchema"]["type"], "object",
        "a client is told what it is being answered with"
    );

    for tool in &tools {
        assert!(
            tool["description"]
                .as_str()
                .is_some_and(|description| !description.is_empty()),
            "{} says what it is for",
            tool["name"]
        );
        assert_eq!(
            tool["inputSchema"]["type"], "object",
            "{} takes an object of arguments",
            tool["name"]
        );
    }

    // The description does not carry the typo an earlier one had.
    let jobs = tools
        .iter()
        .find(|tool| tool["name"] == "show_premises_jobs_by_id")
        .expect("the collections");

    assert!(
        !jobs["description"]
            .as_str()
            .is_some_and(|it| it.contains("Retrives")),
        "{:?}",
        jobs["description"]
    );
}

#[tokio::test]
async fn a_postcode_search_lists_the_addresses_and_hands_back_their_ids() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;
    let numbered = premise(&server.pool, &slot, 0, Some("10")).await;
    premise(&server.pool, &slot, 1, Some("9")).await;

    let mcp = server.mcp().await;
    let result = mcp
        .call(
            "search_premises_from_postcode",
            json!({ "postcode": slot.postcode.replace(' ', "").to_lowercase() }),
        )
        .await;

    assert_ne!(result["isError"], true, "{result}");

    assert_eq!(
        text_of(&result),
        vec![
            format!(
                "{}: 9, TEST STREET, TEST LOCALITY, LEEDS, {}",
                slot.first_id + 1,
                slot.postcode
            ),
            format!(
                "{numbered}: 10, TEST STREET, TEST LOCALITY, LEEDS, {}",
                slot.postcode
            ),
        ],
        "an address on one line, prefixed with the id that finds it again"
    );

    let found = &result["structuredContent"]["premises"];

    assert_eq!(found[0]["id"], slot.first_id + 1);
    assert_eq!(found[0]["addressPostcode"], slot.postcode);
    assert_eq!(found[1]["id"], numbered);
}

#[tokio::test]
async fn a_postcode_with_nothing_at_it_is_a_tool_error() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;

    let result = server
        .mcp()
        .await
        .call(
            "search_premises_from_postcode",
            json!({ "postcode": slot.postcode }),
        )
        .await;

    assert_eq!(result["isError"], true, "{result}");
    assert_eq!(
        text_of(&result),
        vec![format!(
            "No address matching with postcode {} found",
            slot.postcode
        )]
    );
    assert_eq!(
        result["structuredContent"],
        Value::Null,
        "an answer that failed has nothing to structure"
    );
}

#[tokio::test]
async fn the_collections_are_listed_with_where_each_one_sits() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;
    let premises = premise(&server.pool, &slot, 0, Some("6")).await;

    // Yesterday, today and next week, as the server would count them.
    let today = chrono::Local::now().date_naive();
    let (yesterday, next_week) = (
        today - chrono::Duration::days(1),
        today + chrono::Duration::days(7),
    );
    jobs(
        &server.pool,
        premises,
        &[
            ("GREEN", &yesterday.to_string()),
            ("BLACK", &today.to_string()),
            ("GENERAL WASTE", &next_week.to_string()),
        ],
    )
    .await;

    let result = server
        .mcp()
        .await
        .call(
            "show_premises_jobs_by_id",
            json!({ "premisesId": premises }),
        )
        .await;

    assert_ne!(result["isError"], true, "{result}");

    assert_eq!(
        text_of(&result),
        vec![
            format!(
                "Full Address:\n6\nTEST STREET\nTEST LOCALITY\nLEEDS\n{}",
                slot.postcode
            ),
            format!("{yesterday} (Expired): GREEN bin"),
            format!("{today} (Today): BLACK bin"),
            format!("{next_week} (Upcoming): GENERAL WASTE bin"),
        ]
    );

    let collections = &result["structuredContent"];

    assert_eq!(
        collections["addressStreet"], "TEST STREET",
        "the address is structured as it is on the API, rather than nested"
    );
    assert_eq!(
        collections["jobs"],
        json!([
            { "bin": "GREEN", "date": yesterday.to_string(), "status": "Expired" },
            { "bin": "BLACK", "date": today.to_string(), "status": "Today" },
            { "bin": "GENERAL WASTE", "date": next_week.to_string(), "status": "Upcoming" },
        ])
    );
}

#[tokio::test]
async fn a_premises_id_nobody_has_is_a_tool_error() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;

    let mcp = server.mcp().await;

    // An id nothing was ever given out under, and then one too large to be an id
    // at all: both are addresses this site does not have.
    for premises_id in [u32::try_from(slot.first_id + 50).expect("an id"), u32::MAX] {
        let result = mcp
            .call(
                "show_premises_jobs_by_id",
                json!({ "premisesId": premises_id }),
            )
            .await;

        assert_eq!(
            result["isError"], true,
            "premisesId={premises_id}: {result}"
        );
        assert_eq!(
            text_of(&result),
            vec![format!(
                "Address not found (bad premisesId of {premises_id})"
            )]
        );
    }
}

#[tokio::test]
async fn the_permalink_points_at_the_premises_page() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;
    let premises = premise(&server.pool, &slot, 0, Some("6")).await;

    let result = server
        .mcp()
        .await
        .call("get_premises_permalink", json!({ "premisesId": premises }))
        .await;

    assert_ne!(result["isError"], true, "{result}");

    let link = format!("https://bins.felixyeung.com/premises?id={premises}");

    assert_eq!(
        text_of(&result),
        vec![format!("Permalink to the premises page:\n{link}")]
    );
    assert_eq!(result["structuredContent"]["link"], link);
}

#[tokio::test]
async fn a_permalink_for_an_address_this_site_does_not_have_is_a_tool_error() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;

    let result = server
        .mcp()
        .await
        .call(
            "get_premises_permalink",
            json!({ "premisesId": slot.first_id + 50 }),
        )
        .await;

    assert_eq!(result["isError"], true, "{result}");
    assert_eq!(
        text_of(&result),
        vec![format!(
            "Address not found (bad premisesId of {})",
            slot.first_id + 50
        )]
    );
}

#[tokio::test]
async fn a_premises_id_that_cannot_be_one_is_refused_before_the_query() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    // The argument does not match the schema the tool published, so it is refused
    // where it is read rather than looked up: the tool takes the id as an
    // unsigned number.
    let result = server
        .mcp()
        .await
        .call("show_premises_jobs_by_id", json!({ "premisesId": -1 }))
        .await;

    assert_eq!(result["isError"], true, "{result}");
    assert!(
        text_of(&result)[0].contains("expected u32"),
        "the refusal says why: {result}"
    );
    assert_eq!(result["structuredContent"], Value::Null);
}

#[tokio::test]
async fn a_session_can_be_closed_and_is_then_forgotten() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    let mcp = server.mcp().await;

    let closed = mcp
        .client
        .delete(format!("{}/api/mcp", server.base))
        .header("mcp-session-id", &mcp.session)
        .header("mcp-protocol-version", &mcp.version)
        .send()
        .await
        .expect("closing the session");

    assert_eq!(closed.status(), StatusCode::ACCEPTED);

    // Anything else made in that session is a session the server no longer has.
    let response = mcp
        .client
        .post(format!("{}/api/mcp", server.base))
        .header(
            reqwest::header::ACCEPT,
            "application/json, text/event-stream",
        )
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header("mcp-session-id", &mcp.session)
        .header("mcp-protocol-version", &mcp.version)
        .json(&json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/list", "params": {} }))
        .send()
        .await
        .expect("asking in a session that was closed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_client_may_open_the_stream_of_events_for_its_session() {
    let Some(server) = TestServer::start().await else {
        return;
    };

    let mcp = server.mcp().await;

    // Read as the headers rather than the body, since the stream stays open for
    // as long as the session does.
    let stream = mcp
        .client
        .get(format!("{}/api/mcp", server.base))
        .header(reqwest::header::ACCEPT, "text/event-stream")
        .header("mcp-session-id", &mcp.session)
        .header("mcp-protocol-version", &mcp.version)
        .send()
        .await
        .expect("opening the event stream");

    assert_eq!(stream.status(), StatusCode::OK);
    assert!(
        stream
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("text/event-stream")),
        "a session is served as an event stream"
    );
}
