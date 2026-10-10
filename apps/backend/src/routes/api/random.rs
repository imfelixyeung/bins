//! `GET /api/random/premises`: one premise chosen at random.

use axum::Json;
use axum::extract::State;
use sqlx::PgPool;

use crate::search;

use super::{ApiError, Envelope, RandomBody};

/// `GET /api/random/premises`
///
/// Backs the "Suprise Me" button. It returns the whole address, so the payload
/// matches `/api/premises` rather than being a shape of its own.
///
/// An empty database is not an error: `data` is `null`.
pub async fn handler(State(pool): State<PgPool>) -> Result<RandomBody, ApiError> {
    let data = search::random_premise(&pool).await?;

    Ok(Json(Envelope::ok(data)))
}
