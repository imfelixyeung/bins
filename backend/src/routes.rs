use axum::{Router, response::Html, routing::get};
use sqlx::PgPool;

use crate::mcp;

mod api;

pub fn router(pool: PgPool) -> Router {
    api::router()
        .route("/", get(handler))
        // The MCP tools are a service of their own rather than a handler, since
        // the SDK serves them over sessions and events of its own.
        .nest_service("/api/mcp", mcp::service(pool.clone()))
        .with_state(pool)
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
