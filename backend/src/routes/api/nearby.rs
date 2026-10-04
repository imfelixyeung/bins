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
/// The Next.js app built this out of two calls: postcodes.io for the neighbours,
/// then the database for their collections. Everything it needed is in the
/// database now, so the postcode is looked up locally and the neighbours are
/// found with PostGIS.
///
/// A postcode the `sync postcodes` job has not reached yet has no coordinates to
/// search from, and answers with `data: null` rather than a 404. That is what
/// the tRPC procedure returned when it found nothing to draw, so the map in the
/// web app already handles it.
pub async fn handler(
    State(pool): State<PgPool>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<NearbyBody, ApiError> {
    let postcode = postcode(&query).map_err(|issue| ApiError::Invalid(vec![issue]))?;

    let data = search::nearby_postcodes(&pool, postcode).await?;

    Ok(Json(Envelope::ok(data)))
}
