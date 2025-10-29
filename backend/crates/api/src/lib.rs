use axum::{routing::get, Router};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub mod auth;
pub mod handlers;

pub fn create_app() -> Router {
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/users/me", get(handlers::users::get_me))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                ),
        )
}
