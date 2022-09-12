mod config;
mod db;
mod errors;
mod files;
mod logger;
mod models;
mod pagination;
mod routes;

use crate::config::UPLOADS_ENDPOINT;
use axum::{
    handler::Handler,
    http::{header, Request},
    routing::get,
    Router,
};
use std::time::Duration;
use tower_http::sensitive_headers::SetSensitiveHeadersLayer;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::Span;

/// Main application, called by the execution of the software
#[tokio::main]
async fn main() {
    let app = create_app().await;

    // By default the server is bind at "127.0.0.1:3000"
    let addr = std::env::var("ALLOWED_HOST").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    tracing::info!("Listening on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Create the app: setup everything and returns a `Router`
async fn create_app() -> Router {
    logger::setup();
    let _ = db::setup().await;

    let api_routes = Router::new()
        .nest("/users", routes::user::create_route())
        .nest("/auth", routes::auth::create_route())
        .nest("/models", routes::model::create_route());

    Router::new()
        .route(
            &format!("{}/:id", UPLOADS_ENDPOINT),
            get(crate::files::show_uploads),
        )
        // Map all routes to `/v1/*` namespace
        .nest("/v1", api_routes)
        .fallback(crate::routes::page_404.into_service())
        // Mark the `Authorization` request header as sensitive so it doesn't
        // show in logs.
        .layer(SetSensitiveHeadersLayer::new(std::iter::once(
            header::AUTHORIZATION,
        )))
        // Use a layer for `TraceLayer`
        .layer(
            TraceLayer::new_for_http()
                .on_request(|request: &Request<_>, _span: &Span| {
                    tracing::info!("{} {}", request.method(), request.uri());
                })
                .on_failure(
                    |error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                        tracing::error!("{} | {} s", error, latency.as_secs());
                    },
                ),
        )
}
