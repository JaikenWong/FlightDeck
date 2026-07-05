use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum EventType {
    SessionStarted,
    SessionEnded,
    Prompt,
    ReadFile,
    WriteFile,
    DeleteFile,
    RenameFile,
    ShellStart,
    ShellEnd,
    ToolCall,
    ToolResult,
    TestPassed,
    TestFailed,
    Error,
    Warning,
    GitCommit,
    Notification,
}

/// Main event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event ID
    pub id: Uuid,
    /// Session ID this event belongs to
    pub session_id: String,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Type of event
    pub event_type: EventType,
    /// Event payload (agent-specific data)
    pub payload: serde_json::Value,
}

impl Event {
    pub fn new(session_id: String, event_type: EventType, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            timestamp: Utc::now(),
            event_type,
            payload,
        }
    }
}

/// Agent types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Claude,
    Codex,
    Gemini,
    OpenCode,
    Aider,
    Cline,
    Continue,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Claude => write!(f, "claude"),
            AgentType::Codex => write!(f, "codex"),
            AgentType::Gemini => write!(f, "gemini"),
            AgentType::OpenCode => write!(f, "opencode"),
            AgentType::Aider => write!(f, "aider"),
            AgentType::Cline => write!(f, "cline"),
            AgentType::Continue => write!(f, "continue"),
        }
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(AgentType::Claude),
            "codex" => Ok(AgentType::Codex),
            "gemini" => Ok(AgentType::Gemini),
            "opencode" => Ok(AgentType::OpenCode),
            "aider" => Ok(AgentType::Aider),
            "cline" => Ok(AgentType::Cline),
            "continue" => Ok(AgentType::Continue),
            _ => Err(format!("Unknown agent type: {}", s)),
        }
    }
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID
    pub id: String,
    /// Type of agent
    pub agent_type: AgentType,
    /// Project path
    pub project_path: Option<String>,
    /// Model used
    pub model: Option<String>,
    /// Git branch
    pub branch: Option<String>,
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Session end time
    pub ended_at: Option<DateTime<Utc>>,
    /// Duration in milliseconds
    pub duration_ms: Option<i64>,
    /// Total event count
    pub event_count: i64,
    /// Number of files changed
    pub file_count: i64,
    /// Number of shell commands
    pub command_count: i64,
    /// Number of failures
    pub failure_count: i64,
}

impl Session {
    pub fn new(id: String, agent_type: AgentType) -> Self {
        Self {
            id,
            agent_type,
            project_path: None,
            model: None,
            branch: None,
            started_at: Utc::now(),
            ended_at: None,
            duration_ms: None,
            event_count: 0,
            file_count: 0,
            command_count: 0,
            failure_count: 0,
        }
    }

    pub fn finish(&mut self) {
        let now = Utc::now();
        self.ended_at = Some(now);
        self.duration_ms = Some((now - self.started_at).num_milliseconds());
    }
}

/// Session summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub agent_type: AgentType,
    pub project_path: Option<String>,
    pub model: Option<String>,
    pub started_at: DateTime<Utc>,
    pub duration_ms: Option<i64>,
    pub event_count: i64,
    pub failure_count: i64,
}

/// Metrics aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub total_sessions: i64,
    pub total_events: i64,
    pub total_files: i64,
    pub total_commands: i64,
    pub total_failures: i64,
    pub avg_duration_ms: f64,
}