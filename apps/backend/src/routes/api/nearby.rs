//! `GET /api/nearby`: the postcodes around a postcode, and what is collected
//! at each of them.

use std::collections::HashMap;

use axum::Json;
use axum::extract::{Query, State};
use sqlx::PgPool;

use crate::search;

use super::{ApiError, Envelope, NearbyBody, postcode};

/// `GET /api/nearby?postcode=<postcode>`
///
/// Everything needed is in the database, so the postcode is looked up locally
/// and the neighbours are found with PostGIS, rather than calling out to
/// postcodes.io.
///
/// A postcode the `sync postcodes` job has not reached yet has no coordinates to
/// search from, and answers with `data: null` rather than a 404, which the map
/// in the web app already handles.
pub async fn handler(
    State(pool): State<PgPool>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<NearbyBody, ApiError> {
    let postcode = postcode(&query).map_err(|issue| ApiError::Invalid(vec![issue]))?;

    let data = search::nearby_postcodes(&pool, postcode).await?;

    Ok(Json(Envelope::ok(data)))
}
