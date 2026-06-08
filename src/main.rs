mod docs;
mod fake_gen;
mod initialize;
mod mock_router;
mod state;

use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut args = pico_args::Arguments::from_env();
    let port: u16 = args
        .opt_value_from_str("--port")
        .unwrap_or(None)
        .or_else(|| std::env::var("PORT").ok().and_then(|v| v.parse().ok()))
        .unwrap_or(3000);

    let state = Arc::new(RwLock::new(AppState::default()));

    let app = Router::new()
        .route("/_initialize", post(initialize::handle_initialize))
        .route("/_spec", get(docs::handle_spec))
        .route("/_docs", get(docs::handle_docs))
        .with_state(state.clone())
        .fallback(mock_router::fallback_handler(state));

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on http://{}", addr);
    tracing::info!("API docs at http://{addr}/_docs");
    axum::serve(listener, app).await.unwrap();
}
