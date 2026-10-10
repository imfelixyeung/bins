//! `GET /api/datasets`: how fresh each dataset is, how big it is upstream and
//! in the local mirror, and how far its last sync got. Plus
//! `GET /api/datasets/{target}`, listing every sync run of one dataset.

use axum::Json;
use axum::extract::{Path, State};
use sqlx::PgPool;

use crate::datasets;
use crate::sync_run;

use super::{ApiError, Envelope};

/// A shorthand for the payload of `/api/datasets`, where `data` is the datasets.
pub type DatasetsBody = Json<Envelope<datasets::Datasets>>;

/// A shorthand for the payload of `/api/datasets/{target}`, where `data` is the
/// run history.
pub type HistoryBody = Json<Envelope<Vec<sync_run::Run>>>;

/// `GET /api/datasets`
///
/// Backs the datasets page.
///
/// A dataset that has never been synced is answered with null fields rather than
/// left out, so the page can list every dataset it knows about from one response.
pub async fn handler(State(pool): State<PgPool>) -> Result<DatasetsBody, ApiError> {
    let data = datasets::read(&pool).await?;

    Ok(Json(Envelope::ok(data)))
}

/// `GET /api/datasets/{target}`
///
/// Backs a dataset's history page. The target is checked against the datasets
/// this build knows, so `postcodes` is not answered as though it had a history
/// page of its own.
pub async fn history(
    State(pool): State<PgPool>,
    Path(target): Path<String>,
) -> Result<HistoryBody, ApiError> {
    if !datasets::knows(&target) {
        return Err(ApiError::NotFound("Dataset not found"));
    }

    let data = sync_run::history(&pool, &target).await?;

    Ok(Json(Envelope::ok(data)))
}
