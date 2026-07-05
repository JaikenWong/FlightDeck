use async_trait::async_trait;
use serde_json::Value;
use tracing::info;

use flightdeck_core::{Event, EventType, AgentType};
use flightdeck_collector::AgentAdapter;

pub struct CodexAdapter {
    session_id: Option<String>,
    active: bool,
}

impl CodexAdapter {
    pub fn new() -> Self {
        Self {
            session_id: None,
            active: false,
        }
    }
}

#[async_trait]
impl AgentAdapter for CodexAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Codex
    }

    fn name(&self) -> &str {
        "codex"
    }

    async fn start(&mut self, session_id: String) -> anyhow::Result<()> {
        self.session_id = Some(session_id);
        self.active = true;
        info!("Codex adapter started");
        Ok(())
    }

    async fn stop(&mut self) -> anyhow::Result<()> {
        self.session_id = None;
        self.active = false;
        info!("Codex adapter stopped");
        Ok(())
    }

    async fn parse(&self, input: &str) -> anyhow::Result<Vec<Event>> {
        let session_id = self.session_id.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;

        let mut events = Vec::new();

        // Parse Codex-specific format
        if let Ok(payload) = serde_json::from_str::<Value>(input) {
            let event_type = match payload.get("type").and_then(|t| t.as_str()) {
                Some("command") => Some(EventType::ShellStart),
                Some("file_read") => Some(EventType::ReadFile),
                Some("file_write") => Some(EventType::WriteFile),
                Some("error") => Some(EventType::Error),
                _ => Some(EventType::ToolCall),
            };

            if let Some(event_type) = event_type {
                events.push(Event::new(session_id.clone(), event_type, payload));
            }
        }

        Ok(events)
    }

    async fn emit(&self, event: Event) -> anyhow::Result<()> {
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