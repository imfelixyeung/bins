//! `GET /api/sitemaps/premises.xml`: the premises sitemaps.
//!
//! Without a `page` the response is the index of every paged sitemap; with one
//! it is that page's addresses. Both are XML rather than the usual envelope, as
//! they are for crawlers rather than the app.

use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::sitemap;

use super::{ApiError, sitemap_page};

/// How long a response may be reused. The Next.js app cached these for a minute
/// before serving them again.
const CACHE_CONTROL_VALUE: &str = "public, max-age=60";

/// `GET /api/sitemaps/premises.xml` and `GET /api/sitemaps/premises.xml?page=<page>`
pub async fn handler(
    State(pool): State<PgPool>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    let site = sitemap::site();

    let xml = match sitemap_page(&query).map_err(|issue| ApiError::Invalid(vec![issue]))? {
        Some(page) => sitemap::urlset(&site, &sitemap::page(&pool, page).await?),
        None => sitemap::index(&site, sitemap::pages(&pool).await?),
    };

    Ok((
        [
            (CONTENT_TYPE, "application/xml"),
            (CACHE_CONTROL, CACHE_CONTROL_VALUE),
        ],
        xml,
    )
        .into_response())
}
