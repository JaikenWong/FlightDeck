use axum::{extract::{Path, State}, Json};
use std::sync::Arc;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use flightdeck_collector::Collector;
use flightdeck_core::{Session, SessionSummary, Event, Metrics};
use flightdeck_parser::{ClaudeSessionParser, CodexSessionParser};

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

/// Import all Claude sessions from ~/.claude/projects/
pub async fn import_claude_sessions(
    State(collector): State<Arc<Collector>>,
) -> Json<ApiResponse<ImportResult>> {
    let claude_dir = dirs::home_dir()
        .map(|h| h.join(".claude"))
        .unwrap_or_else(|| PathBuf::from("~/.claude"));

    let parser = ClaudeSessionParser::new(claude_dir);

    match parser.parse_all_sessions() {
        Ok(parsed) => {
            let mut imported = 0;
            let mut errors = 0;

            for ps in &parsed {
                if let Err(e) = collector.storage().create_session(&ps.session) {
                    tracing::error!("Failed to create session {}: {}", ps.session.id, e);
                    errors += 1;
                    continue;
                }

                for event in &ps.events {
                    if let Err(e) = collector.storage().insert_event(event) {
                        tracing::error!("Failed to insert event: {}", e);
                        errors += 1;
                    }
                }

                imported += 1;
            }

            tracing::info!("Imported {} Claude sessions", imported);
            Json(ApiResponse::ok(ImportResult {
                total_found: parsed.len(),
                imported,
                errors,
            }))
        }
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}

/// Import all Codex sessions from ~/.codex/sessions/
pub async fn import_codex_sessions(
    State(collector): State<Arc<Collector>>,
) -> Json<ApiResponse<ImportResult>> {
    let codex_dir = dirs::home_dir()
        .map(|h| h.join(".codex"))
        .unwrap_or_else(|| PathBuf::from("~/.codex"));

    let parser = CodexSessionParser::new(codex_dir);

    match parser.parse_all_sessions() {
        Ok(parsed) => {
            let mut imported = 0;
            let mut errors = 0;

            for ps in &parsed {
                if let Err(e) = collector.storage().create_session(&ps.session) {
                    tracing::error!("Failed to create session {}: {}", ps.session.id, e);
                    errors += 1;
                    continue;
                }

                for event in &ps.events {
                    if let Err(e) = collector.storage().insert_event(event) {
                        tracing::error!("Failed to insert event: {}", e);
                        errors += 1;
                    }
                }

                imported += 1;
            }

            tracing::info!("Imported {} Codex sessions", imported);
            Json(ApiResponse::ok(ImportResult {
                total_found: parsed.len(),
                imported,
                errors,
            }))
        }
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}

#[derive(Serialize)]
pub struct ImportResult {
    pub total_found: usize,
    pub imported: usize,
    pub errors: usize,
}