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
use backend::{db, import::premises::search_postcode, routes};
use chrono::NaiveDate;
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
        let response = reqwest::get(format!("{}{path}", self.base))
            .await
            .unwrap_or_else(|err| panic!("GET {path}: {err}"));

        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();

        let body = response
            .text()
            .await
            .unwrap_or_else(|err| panic!("reading the body of GET {path}: {err}"));
        let body: Value = serde_json::from_str(&body)
            .unwrap_or_else(|err| panic!("GET {path} returned non-JSON {body:?}: {err}"));

        assert_eq!(
            content_type, "application/json",
            "unexpected content type for {path}"
        );
        (status, body)
    }
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
    assert!(
        data["addressPostcode"]
            .as_str()
            .is_some_and(|p| !p.is_empty()),
        "the caller needs a postcode to send the visitor to"
    );
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

    // One degree of latitude is about 111km, so the second postcode is a
    // kilometre away and the third is well outside the two kilometre radius.
    let neighbour = nearby_postcode(&slot, 2);
    let faraway = nearby_postcode(&slot, 3);
    postcode(&server.pool, &slot.postcode, 53.80, -1.56).await;
    postcode(&server.pool, &neighbour, 53.81, -1.56).await;
    postcode(&server.pool, &faraway, 53.90, -1.56).await;

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
    assert_eq!(data[0]["latitude"], 53.80);
    assert_eq!(data[0]["longitude"], -1.56);
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

#[tokio::test]
async fn jobs_serves_json_only_for_now() {
    let Some(server) = TestServer::start().await else {
        return;
    };
    let slot = slot(&server.pool).await;
    let id = premise(&server.pool, &slot, 0, Some("6")).await;

    for format in ["csv", "ical"] {
        let (status, body) = server
            .get(&format!("/api/jobs?premises={id}&format={format}"))
            .await;

        assert_eq!(status, StatusCode::NOT_IMPLEMENTED, "{format}");
        assert_eq!(body["error"], true, "{format}");
        assert!(
            body["message"].as_str().is_some_and(|m| m.contains(format)),
            "{format}: {}",
            body["message"]
        );
    }

    let (status, body) = server
        .get(&format!("/api/jobs?premises={id}&format=xml"))
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["issues"][0]["path"], json!(["format"]));
}
