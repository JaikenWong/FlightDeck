use axum::{extract::{Path, State}, Json};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use flightdeck_collector::Collector;
use flightdeck_core::{Session, SessionSummary, Event, Metrics};

#[derive(Deserialize)]
pub struct ListSessionsQuery {
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    pub fn err(msg: String) -> Self {
        Self { success: false, data: None, error: Some(msg) }
    }
}

pub async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("FlightDeck API is running"))
}

pub async fn list_sessions(
    State(collector): State<Arc<Collector>>,
) -> Json<ApiResponse<Vec<SessionSummary>>> {
    match collector.list_sessions(100) {
        Ok(sessions) => Json(ApiResponse::ok(sessions)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}

pub async fn get_session(
    State(collector): State<Arc<Collector>>,
    Path(session_id): Path<String>,
) -> Json<ApiResponse<Option<Session>>> {
    match collector.get_session(&session_id) {
        Ok(session) => Json(ApiResponse::ok(session)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}

pub async fn get_events(
    State(collector): State<Arc<Collector>>,
    Path(session_id): Path<String>,
) -> Json<ApiResponse<Vec<Event>>> {
    match collector.get_events(&session_id) {
        Ok(events) => Json(ApiResponse::ok(events)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}

pub async fn get_metrics(
    State(collector): State<Arc<Collector>>,
) -> Json<ApiResponse<Metrics>> {
    match collector.get_metrics() {
        Ok(metrics) => Json(ApiResponse::ok(metrics)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}