use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use walkdir::WalkDir;
use tracing::{info, debug};

use flightdeck_core::{Event, EventType, Session, AgentType};
use crate::error::ParserError;

/// Codex session file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexSessionEntry {
    pub timestamp: Option<String>,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub payload: Option<serde_json::Value>,
}

/// Parsed session with events
#[derive(Debug, Clone)]
pub struct ParsedSession {
    pub session: Session,
    pub events: Vec<Event>,
}

/// Parser for Codex CLI session files
pub struct CodexSessionParser {
    codex_dir: PathBuf,
}

impl CodexSessionParser {
    pub fn new(codex_dir: PathBuf) -> Self {
        Self { codex_dir }
    }

    /// Discover all session files
    pub fn discover_sessions(&self) -> Result<Vec<PathBuf>, ParserError> {
        let mut sessions = Vec::new();
        let sessions_dir = self.codex_dir.join("sessions");

        if !sessions_dir.exists() {
            return Ok(sessions);
        }

        for entry in WalkDir::new(&sessions_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "jsonl") {
                sessions.push(path.to_path_buf());
            }
        }

        info!("Discovered {} Codex session files", sessions.len());
        Ok(sessions)
    }

    /// Parse a single session file
    pub fn parse_session_file(&self, path: &Path) -> Result<ParsedSession, ParserError> {
        let content = std::fs::read_to_string(path)?;
        let mut entries: Vec<CodexSessionEntry> = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<CodexSessionEntry>(line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    debug!("Skipping malformed line: {}", e);
                }
            }
        }

        if entries.is_empty() {
            return Err(ParserError::Parse("No valid entries found".to_string()));
        }

        // Find session_meta for metadata
        let session_meta = entries.iter().find(|e| e.entry_type == "session_meta");

        let session_id = session_meta
            .and_then(|e| e.payload.as_ref())
            .and_then(|p| p.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or(&Uuid::new_v4().to_string())
            .to_string();

        let project_path = session_meta
            .and_then(|e| e.payload.as_ref())
            .and_then(|p| p.get("cwd"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let cli_version = session_meta
            .and_then(|e| e.payload.as_ref())
            .and_then(|p| p.get("cli_version"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Parse timestamps
        let started_at = session_meta
            .and_then(|e| e.payload.as_ref())
            .and_then(|p| p.get("timestamp"))
            .and_then(|v| v.as_str())
            .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| {
                entries.first()
                    .and_then(|e| e.timestamp.as_ref())
                    .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|| Utc::now())
            });

        let ended_at = entries.last()
            .and_then(|e| e.timestamp.as_ref())
            .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let duration_ms = ended_at.map(|end| (end - started_at).num_milliseconds());

        // Convert entries to events
        let mut events = Vec::new();
        let mut file_count = 0i64;
        let mut command_count = 0i64;
        let mut failure_count = 0i64;

        for entry in &entries {
            let timestamp = entry.timestamp.as_ref()
                .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now());

            match entry.entry_type.as_str() {
                "session_meta" => {
                    events.push(Event {
                        id: Uuid::new_v4(),
                        session_id: session_id.clone(),
                        timestamp,
                        event_type: EventType::SessionStarted,
                        payload: entry.payload.clone().unwrap_or_default(),
                    });
                }
                "response_item" => {
                    if let Some(payload) = &entry.payload {
                        let item_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");

                        match item_type {
                            "message" => {
                                let role = payload.get("role").and_then(|v| v.as_str()).unwrap_or("unknown");
                                let content = payload.get("content")
                                    .and_then(|c| c.as_array())
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|item| {
                                                let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                                match item_type {
                                                    "input_text" | "output_text" => {
                                                        item.get("text").and_then(|v| v.as_str()).map(|s| s.to_string())
                                                    }
                                                    "reasoning" => {
                                                        item.get("reasoning").and_then(|v| v.as_str()).map(|s| s.to_string())
                                                    }
                                                    _ => None,
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                            .join("\n")
                                    })
                                    .unwrap_or_default();

                                if !content.is_empty() {
                                    events.push(Event {
                                        id: Uuid::new_v4(),
                                        session_id: session_id.clone(),
                                        timestamp,
                                        event_type: EventType::Prompt,
                                        payload: serde_json::json!({
                                            "role": role,
                                            "text": content,
                                            "model": "codex"
                                        }),
                                    });
                                }
                            }
                            "function_call" => {
                                let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                                let args = payload.get("arguments").cloned().unwrap_or_default();

                                let event_type = match name {
                                    "shell" | "exec" => {
                                        command_count += 1;
                                        EventType::ShellStart
                                    }
                                    "read_file" => {
                                        file_count += 1;
                                        EventType::ReadFile
                                    }
                                    "write_file" | "apply_patch" => {
                                        file_count += 1;
                                        EventType::WriteFile
                                    }
                                    "list_directory" | "glob" | "grep" => EventType::ToolCall,
                                    _ => EventType::ToolCall,
                                };

                                events.push(Event {
                                    id: Uuid::new_v4(),
                                    session_id: session_id.clone(),
                                    timestamp,
                                    event_type,
                                    payload: serde_json::json!({
                                        "tool": name,
                                        "input": args
                                    }),
                                });
                            }
                            "function_call_output" => {
                                let output = payload.get("output").cloned().unwrap_or_default();
                                let is_error = output.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);

                                if is_error {
                                    failure_count += 1;
                                }

                                events.push(Event {
                                    id: Uuid::new_v4(),
                                    session_id: session_id.clone(),
                                    timestamp,
                                    event_type: if is_error { EventType::Error } else { EventType::ToolResult },
                                    payload: serde_json::json!({
                                        "result": output,
                                        "is_error": is_error
                                    }),
                                });
                            }
                            "reasoning" => {
                                let reasoning = payload.get("reasoning").and_then(|v| v.as_str()).unwrap_or("");
                                if !reasoning.is_empty() {
                                    events.push(Event {
                                        id: Uuid::new_v4(),
                                        session_id: session_id.clone(),
                                        timestamp,
                                        event_type: EventType::Notification,
                                        payload: serde_json::json!({
                                            "kind": "thinking",
                                            "content": reasoning
                                        }),
                                    });
                                }
                            }
                            _ => {
                                debug!("Skipping response_item type: {}", item_type);
                            }
                        }
                    }
                }
                "event_msg" => {
                    if let Some(payload) = &entry.payload {
                        let msg_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");

                        match msg_type {
                            "task_started" => {
                                events.push(Event {
                                    id: Uuid::new_v4(),
                                    session_id: session_id.clone(),
                                    timestamp,
                                    event_type: EventType::Notification,
                                    payload: serde_json::json!({
                                        "kind": "task_started",
                                        "data": payload
                                    }),
                                });
                            }
                            "turn_completed" => {
                                let duration = payload.get("duration_ms").and_then(|v| v.as_i64());
                                events.push(Event {
                                    id: Uuid::new_v4(),
                                    session_id: session_id.clone(),
                                    timestamp,
                                    event_type: EventType::Notification,
                                    payload: serde_json::json!({
                                        "kind": "turn_completed",
                                        "duration_ms": duration
                                    }),
                                });
                            }
                            "turn_aborted" => {
                                failure_count += 1;
                                events.push(Event {
                                    id: Uuid::new_v4(),
                                    session_id: session_id.clone(),
                                    timestamp,
                                    event_type: EventType::Warning,
                                    payload: serde_json::json!({
                                        "kind": "turn_aborted",
                                        "reason": payload.get("reason").and_then(|v| v.as_str())
                                    }),
                                });
                            }
                            _ => {
                                debug!("Skipping event_msg type: {}", msg_type);
                            }
                        }
                    }
                }
                "turn_context" => {
                    // Skip turn_context for now
                    debug!("Skipping turn_context");
                }
                _ => {
                    debug!("Skipping entry type: {}", entry.entry_type);
                }
            }
        }

        let session = Session {
            id: session_id,
            agent_type: AgentType::Codex,
            project_path,
            model: cli_version, // Use CLI version as model info
            branch: None,
            started_at,
            ended_at,
            duration_ms,
            event_count: events.len() as i64,
            file_count,
            command_count,
            failure_count,
        };

        Ok(ParsedSession { session, events })
    }

    /// Parse all sessions from the Codex directory
    pub fn parse_all_sessions(&self) -> Result<Vec<ParsedSession>, ParserError> {
        let paths = self.discover_sessions()?;
        let mut sessions = Vec::new();

        for path in paths {
            match self.parse_session_file(&path) {
                Ok(session) => sessions.push(session),
                Err(e) => {
                    debug!("Failed to parse {}: {}", path.display(), e);
                }
            }
        }

        // Sort by start time (newest first)
        sessions.sort_by(|a, b| b.session.started_at.cmp(&a.session.started_at));

        info!("Parsed {} Codex sessions", sessions.len());
        Ok(sessions)
    }
}