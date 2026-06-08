mod state;
mod initialize;
mod mock_router;
mod fake_gen;
mod docs;

use axum::{Router, routing::{get, post}};
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

    let state = Arc::new(RwLock::new(AppState::default()));

    let app = Router::new()
        .route("/_initialize", post(initialize::handle_initialize))
        .route("/_spec", get(docs::handle_spec))
        .route("/_docs", get(docs::handle_docs))
        .with_state(state.clone())
        .fallback(mock_router::fallback_handler(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Listening on http://0.0.0.0:3000");
    tracing::info!("API docs at http://0.0.0.0:3000/_docs");
    axum::serve(listener, app).await.unwrap();
}
