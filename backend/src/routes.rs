use axum::{Router, response::Html, routing::get};
use sqlx::PgPool;

mod api;

pub fn router(pool: PgPool) -> Router {
    api::router().route("/", get(handler)).with_state(pool)
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
