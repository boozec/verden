use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Setup tracing subscriber logger
pub fn setup() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "m6_ie_2022=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
