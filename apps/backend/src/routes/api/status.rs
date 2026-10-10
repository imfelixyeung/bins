//! `GET /api/status`: how fresh each dataset is, and how far its last sync got.

use axum::Json;
use axum::extract::State;
use sqlx::PgPool;

use crate::status;

use super::{ApiError, Envelope};

/// A shorthand for the payload of `/api/status`, where `data` is the status.
pub type StatusBody = Json<Envelope<status::Status>>;

/// `GET /api/status`
///
/// Backs the status page.
///
/// A dataset that has never been synced is answered with null fields rather than
/// left out, so the page can list every dataset it knows about from one response.
pub async fn handler(State(pool): State<PgPool>) -> Result<StatusBody, ApiError> {
    let data = status::read(&pool).await?;

    Ok(Json(Envelope::ok(data)))
}
