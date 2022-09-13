mod config;
mod db;
mod errors;
mod files;
mod logger;
mod models;
mod pagination;
mod routes;

use crate::config::CONFIG;
use axum::{
    handler::Handler,
    http::{header, Method, Request},
    routing::get,
    Router,
};

use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;
use tower_http::{
    classify::ServerErrorsFailureClass,
    cors::{Any, CorsLayer},
    sensitive_headers::SetSensitiveHeadersLayer,
    trace::TraceLayer,
};
use tracing::Span;

/// Main application, called by the execution of the software
#[tokio::main]
async fn main() {
    let app = create_app().await;

    let host = &CONFIG.allowed_host;
    let addr = match host.parse::<SocketAddr>() {
        Ok(addr) => addr,
        Err(_) => match host.to_socket_addrs() {
            Ok(mut addr) => addr.next().unwrap(),
            Err(e) => {
                panic!("{}", e);
            }
        },
    };
    tracing::info!("Listening on {}", addr);

    axum::Server::bind(&addr)
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
            &format!("{}/:id", CONFIG.uploads_endpoint),
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
        .layer(
            CorsLayer::new()
                .allow_methods([
                    Method::OPTIONS,
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                ])
                .allow_headers(vec![header::CONTENT_TYPE])
                .allow_origin(Any),
        )
}
