mod routes;

use axum::{routing::get, Router};
use tower_http::cors::CorsLayer;
use tracing_subscriber;
use std::sync::Arc;

use flightdeck_storage::Storage;
use flightdeck_collector::Collector;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let storage = Storage::new("flightdeck.db")?;
    let collector = Arc::new(Collector::new(storage));

    let app = Router::new()
        .route("/api/sessions", get(routes::list_sessions))
        .route("/api/sessions/:id", get(routes::get_session))
        .route("/api/sessions/:id/events", get(routes::get_events))
        .route("/api/metrics", get(routes::get_metrics))
        .route("/api/health", get(routes::health))
        .layer(CorsLayer::permissive())
        .with_state(collector);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    tracing::info!("FlightDeck API listening on 0.0.0.0:3001");
    axum::serve(listener, app).await?;

    Ok(())
}