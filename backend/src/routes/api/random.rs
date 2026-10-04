//! `GET /api/random/premises`: one premise chosen at random.

use axum::Json;
use axum::extract::State;
use sqlx::PgPool;

use crate::search;

use super::{ApiError, Envelope, RandomBody};

/// `GET /api/random/premises`
///
/// Backs the "Suprise Me" button. The tRPC procedure it replaces took no input
/// and answered with an id and a postcode, because all the caller wanted was
/// somewhere to send the visitor. This returns the whole address instead, so
/// the payload matches `/api/premises` rather than being a shape of its own.
///
/// An empty database is not an error: `data` is `null`, as `findFirst` made it.
pub async fn handler(State(pool): State<PgPool>) -> Result<RandomBody, ApiError> {
    let data = search::random_premise(&pool).await?;

    Ok(Json(Envelope::ok(data)))
}
