//! `GET /api/premises`: every premise at a postcode.

use std::collections::HashMap;

use axum::Json;
use axum::extract::{Query, State};
use sqlx::PgPool;

use crate::search;

use super::{ApiError, Envelope, PremisesBody, postcode};

/// `GET /api/premises?postcode=<postcode>`
///
/// An unknown postcode is not an error: the endpoint answers with an empty list,
/// exactly as the Next.js route did.
pub async fn handler(
    State(pool): State<PgPool>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<PremisesBody, ApiError> {
    let postcode = postcode(&query).map_err(|issue| ApiError::Invalid(vec![issue]))?;

    let data = search::premises(&pool, postcode).await?;

    Ok(Json(Envelope::ok(data)))
}
