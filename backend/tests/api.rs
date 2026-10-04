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
use chrono::{DateTime, NaiveDate, Utc};
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
    // The whole address, not just the two columns the tRPC procedure returned.
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
        &format!("{site}/api/v2/sitemaps/premises.xml?page=0")
    );
    assert_eq!(
        urls.last().unwrap(),
        &format!(
            "{site}/api/v2/sitemaps/premises.xml?page={}",
            expected_pages - 1
        )
    );
    assert!(
        urls.iter()
            .all(|url| url.starts_with(&format!("{site}/api/v2/"))),
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

    // The Next.js route answered 404 here. An empty sitemap is what the query
    // it called returns, and is easier for a crawler to move past.
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
