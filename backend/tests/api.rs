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
}

async fn slot(pool: &PgPool) -> Slot {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let index = NEXT.fetch_add(1, Ordering::SeqCst) + 1;

    let slot = Slot {
        postcode: format!("ZZ{index:02} 1ZZ"),
        first_id: 9_000_000 + index as i32 * 100,
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

    slot
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
