use flightdeck_core::{Event, AgentType};
use async_trait::async_trait;

/// Trait for agent-specific adapters
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// Get the agent type this adapter handles
    fn agent_type(&self) -> AgentType;

    /// Get the name of this adapter
    fn name(&self) -> &str;

    /// Start collecting events for a session
    async fn start(&mut self, session_id: String) -> anyhow::Result<()>;

    /// Stop collecting events
    async fn stop(&mut self) -> anyhow::Result<()>;

    /// Parse raw input into events
    async fn parse(&self, input: &str) -> anyhow::Result<Vec<Event>>;

    /// Emit an event (for real-time streaming)
    async fn emit(&self, event: Event) -> anyhow::Result<()>;

    /// Check if the adapter is currently active
    fn is_active(&self) -> bool;

    /// Get the current session ID
    fn current_session_id(&self) -> Option<&str>;
}