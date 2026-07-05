use async_trait::async_trait;
use serde_json::Value;
use tracing::info;

use flightdeck_core::{Event, EventType, AgentType};
use flightdeck_collector::AgentAdapter;

pub struct ClaudeAdapter {
    session_id: Option<String>,
    active: bool,
}

impl ClaudeAdapter {
    pub fn new() -> Self {
        Self {
            session_id: None,
            active: false,
        }
    }
}

#[async_trait]
impl AgentAdapter for ClaudeAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Claude
    }

    fn name(&self) -> &str {
        "claude-code"
    }

    async fn start(&mut self, session_id: String) -> anyhow::Result<()> {
        self.session_id = Some(session_id);
        self.active = true;
        info!("Claude adapter started");
        Ok(())
    }

    async fn stop(&mut self) -> anyhow::Result<()> {
        self.session_id = None;
        self.active = false;
        info!("Claude adapter stopped");
        Ok(())
    }

    async fn parse(&self, input: &str) -> anyhow::Result<Vec<Event>> {
        let session_id = self.session_id.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;

        let mut events = Vec::new();

        // Try to parse as JSON (Claude hook output format)
        if let Ok(payload) = serde_json::from_str::<Value>(input) {
            let event_type = match payload.get("type").and_then(|t| t.as_str()) {
                Some("tool_use") => {
                    match payload.get("name").and_then(|n| n.as_str()) {
                        Some("Read") => Some(EventType::ReadFile),
                        Some("Write") | Some("Edit") | Some("MultiEdit") => Some(EventType::WriteFile),
                        Some("Bash") => Some(EventType::ShellStart),
                        Some("Glob") | Some("Grep") => Some(EventType::ToolCall),
                        _ => Some(EventType::ToolCall),
                    }
                }
                Some("tool_result") => {
                    if payload.get("is_error").and_then(|e| e.as_bool()).unwrap_or(false) {
                        Some(EventType::Error)
                    } else {
                        Some(EventType::ToolResult)
                    }
                }
                Some("assistant") => Some(EventType::Prompt),
                Some("user") => Some(EventType::Prompt),
                _ => None,
            };

            if let Some(event_type) = event_type {
                events.push(Event::new(
                    session_id.clone(),
                    event_type,
                    payload,
                ));
            }
        } else {
            // Plain text log line - try to detect patterns
            let event_type = if input.contains("error") || input.contains("Error") {
                Some(EventType::Error)
            } else if input.contains("warning") || input.contains("Warning") {
                Some(EventType::Warning)
            } else {
                None
            };

            if let Some(event_type) = event_type {
                events.push(Event::new(
                    session_id.clone(),
                    event_type,
                    serde_json::json!({ "message": input }),
                ));
            }
        }

        Ok(events)
    }

    async fn emit(&self, event: Event) -> anyhow::Result<()> {
        // In a real implementation, this would push to a channel
        info!("Emitting event: {:?} for session: {}", event.event_type, event.session_id);
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn current_session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}