use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use walkdir::WalkDir;
use tracing::{info, debug};

use flightdeck_core::{Event, EventType, Session, AgentType};
use crate::error::ParserError;

/// Claude session file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSessionEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub timestamp: Option<String>,
    pub uuid: Option<String>,
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    pub git_branch: Option<String>,
    pub message: Option<ClaudeMessage>,
    pub parent_uuid: Option<String>,
    pub is_sidechain: Option<bool>,
}

/// Claude message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessage {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: Option<String>,
    pub role: Option<String>,
    pub model: Option<String>,
    pub content: Option<Vec<ClaudeContent>>,
    pub usage: Option<ClaudeUsage>,
    pub stop_reason: Option<String>,
}

/// Claude content block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClaudeContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "thinking")]
    Thinking { thinking: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: Option<String>,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: Option<String>,
        content: Option<serde_json::Value>,
        is_error: Option<bool>,
    },
}

/// Claude token usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeUsage {
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
}

/// Parsed session with events
#[derive(Debug, Clone)]
pub struct ParsedSession {
    pub session: Session,
    pub events: Vec<Event>,
}

/// Parser for Claude Code session files
pub struct ClaudeSessionParser {
    claude_dir: PathBuf,
}

impl ClaudeSessionParser {
    pub fn new(claude_dir: PathBuf) -> Self {
        Self { claude_dir }
    }

    /// Discover all session files
    pub fn discover_sessions(&self) -> Result<Vec<PathBuf>, ParserError> {
        let mut sessions = Vec::new();
        let projects_dir = self.claude_dir.join("projects");

        if !projects_dir.exists() {
            return Ok(sessions);
        }

        for entry in WalkDir::new(&projects_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "jsonl") {
                // Check if it's a session file (not a subagent file)
                if !path.to_string_lossy().contains("subagents") {
                    sessions.push(path.to_path_buf());
                }
            }
        }

        info!("Discovered {} session files", sessions.len());
        Ok(sessions)
    }

    /// Parse a single session file
    pub fn parse_session_file(&self, path: &Path) -> Result<ParsedSession, ParserError> {
        let content = std::fs::read_to_string(path)?;
        let mut entries: Vec<ClaudeSessionEntry> = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<ClaudeSessionEntry>(line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    debug!("Skipping malformed line: {}", e);
                }
            }
        }

        if entries.is_empty() {
            return Err(ParserError::Parse("No valid entries found".to_string()));
        }

        // Extract session metadata from first entry
        let first = &entries[0];
        let session_id = first.session_id.clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let project_path = first.cwd.clone();
        let branch = first.git_branch.clone();

        // Find model from assistant messages
        let model = entries.iter()
            .filter_map(|e| e.message.as_ref())
            .find_map(|m| m.model.clone());

        // Parse timestamp
        let started_at = first.timestamp.as_ref()
            .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now());

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
                "user" => {
                    if let Some(msg) = &entry.message {
                        if let Some(content) = &msg.content {
                            for block in content {
                                if let ClaudeContent::Text { text } = block {
                                    events.push(Event {
                                        id: Uuid::new_v4(),
                                        session_id: session_id.clone(),
                                        timestamp,
                                        event_type: EventType::Prompt,
                                        payload: serde_json::json!({
                                            "role": "user",
                                            "text": text
                                        }),
                                    });
                                }
                            }
                        }
                    }
                }
                "assistant" => {
                    if let Some(msg) = &entry.message {
                        if let Some(content) = &msg.content {
                            for block in content {
                                match block {
                                    ClaudeContent::Text { text } => {
                                        events.push(Event {
                                            id: Uuid::new_v4(),
                                            session_id: session_id.clone(),
                                            timestamp,
                                            event_type: EventType::Prompt,
                                            payload: serde_json::json!({
                                                "role": "assistant",
                                                "text": text,
                                                "model": msg.model
                                            }),
                                        });
                                    }
                                    ClaudeContent::Thinking { thinking } => {
                                        events.push(Event {
                                            id: Uuid::new_v4(),
                                            session_id: session_id.clone(),
                                            timestamp,
                                            event_type: EventType::Notification,
                                            payload: serde_json::json!({
                                                "kind": "thinking",
                                                "content": thinking
                                            }),
                                        });
                                    }
                                    ClaudeContent::ToolUse { name, input, .. } => {
                                        let event_type = match name.as_str() {
                                            "Read" | "read" => {
                                                file_count += 1;
                                                EventType::ReadFile
                                            }
                                            "Write" | "Edit" | "MultiEdit" | "apply_patch" => {
                                                file_count += 1;
                                                EventType::WriteFile
                                            }
                                            "Bash" | "bash" => {
                                                command_count += 1;
                                                EventType::ShellStart
                                            }
                                            "Glob" | "Grep" | "LS" => EventType::ToolCall,
                                            _ => EventType::ToolCall,
                                        };

                                        events.push(Event {
                                            id: Uuid::new_v4(),
                                            session_id: session_id.clone(),
                                            timestamp,
                                            event_type,
                                            payload: serde_json::json!({
                                                "tool": name,
                                                "input": input
                                            }),
                                        });
                                    }
                                    ClaudeContent::ToolResult { content, is_error, .. } => {
                                        let is_err = is_error.unwrap_or(false);
                                        if is_err {
                                            failure_count += 1;
                                        }

                                        events.push(Event {
                                            id: Uuid::new_v4(),
                                            session_id: session_id.clone(),
                                            timestamp,
                                            event_type: if is_err { EventType::Error } else { EventType::ToolResult },
                                            payload: serde_json::json!({
                                                "result": content,
                                                "is_error": is_err
                                            }),
                                        });
                                    }
                                }
                            }
                        }

                        // Add token usage event
                        if let Some(usage) = &msg.usage {
                            events.push(Event {
                                id: Uuid::new_v4(),
                                session_id: session_id.clone(),
                                timestamp,
                                event_type: EventType::Notification,
                                payload: serde_json::json!({
                                    "kind": "usage",
                                    "input_tokens": usage.input_tokens,
                                    "output_tokens": usage.output_tokens,
                                    "model": msg.model
                                }),
                            });
                        }
                    }
                }
                _ => {
                    debug!("Skipping entry type: {}", entry.entry_type);
                }
            }
        }

        let session = Session {
            id: session_id,
            agent_type: AgentType::Claude,
            project_path,
            model,
            branch,
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

    /// Parse all sessions from the Claude directory
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

        info!("Parsed {} sessions", sessions.len());
        Ok(sessions)
    }
}