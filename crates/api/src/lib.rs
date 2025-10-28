use axum::{routing::get, Router};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

pub mod auth;
pub mod handlers;

pub fn create_app() -> Router {
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
}
