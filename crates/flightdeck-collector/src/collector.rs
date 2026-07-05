use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use flightdeck_core::{Event, EventType, Session, AgentType};
use flightdeck_storage::Storage;

use crate::adapter::AgentAdapter;
use crate::error::CollectorError;

/// Main collector that manages adapters and persists events
pub struct Collector {
    storage: Arc<Storage>,
    adapters: RwLock<HashMap<AgentType, Box<dyn AgentAdapter>>>,
    active_sessions: RwLock<HashMap<String, AgentType>>,
}

impl Collector {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage: Arc::new(storage),
            adapters: RwLock::new(HashMap::new()),
            active_sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Get reference to storage
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// Register an agent adapter
    pub async fn register_adapter(&self, adapter: Box<dyn AgentAdapter>) {
        let agent_type = adapter.agent_type();
        info!("Registering adapter for {:?}", agent_type);
        self.adapters.write().await.insert(agent_type, adapter);
    }

    /// Start a new session for the given agent type
    pub async fn start_session(
        &self,
        agent_type: AgentType,
        project_path: Option<String>,
        model: Option<String>,
        branch: Option<String>,
    ) -> Result<String, CollectorError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let mut session = Session::new(session_id.clone(), agent_type.clone());
        session.project_path = project_path;
        session.model = model;
        session.branch = branch;

        self.storage.create_session(&session)?;

        let mut adapters = self.adapters.write().await;
        if let Some(adapter) = adapters.get_mut(&agent_type) {
            adapter.start(session_id.clone()).await
                .map_err(|e| CollectorError::Adapter(e.to_string()))?;
        }

        self.active_sessions.write().await.insert(session_id.clone(), agent_type.clone());
        info!("Started session: {} for agent: {:?}", session_id, agent_type);
        Ok(session_id)
    }

    /// Stop an active session
    pub async fn stop_session(&self, session_id: &str) -> Result<(), CollectorError> {
        let mut sessions = self.active_sessions.write().await;
        if let Some(agent_type) = sessions.remove(session_id) {
            let mut adapters = self.adapters.write().await;
            if let Some(adapter) = adapters.get_mut(&agent_type) {
                adapter.stop().await
                    .map_err(|e| CollectorError::Adapter(e.to_string()))?;
            }

            if let Some(mut session) = self.storage.get_session(session_id)? {
                session.finish();
                self.storage.update_session(&session)?;
            }

            info!("Stopped session: {}", session_id);
            Ok(())
        } else {
            Err(CollectorError::SessionNotActive)
        }
    }

    /// Record an event for a session
    pub async fn record_event(&self, event: Event) -> Result<(), CollectorError> {
        let session_id = event.session_id.clone();
        let event_type = event.event_type.clone();

        self.storage.insert_event(&event)?;

        // Update session counters
        if let Some(mut session) = self.storage.get_session(&session_id)? {
            session.event_count += 1;

            match event_type {
                EventType::ReadFile | EventType::WriteFile | EventType::DeleteFile | EventType::RenameFile => {
                    session.file_count += 1;
                }
                EventType::ShellStart => {
                    session.command_count += 1;
                }
                EventType::TestFailed | EventType::Error => {
                    session.failure_count += 1;
                }
                _ => {}
            }

            self.storage.update_session(&session)?;
        }

        Ok(())
    }

    /// Process raw input from an agent
    pub async fn process_input(&self, agent_type: &AgentType, input: &str) -> Result<Vec<Event>, CollectorError> {
        let adapters = self.adapters.read().await;
        if let Some(adapter) = adapters.get(agent_type) {
            let events = adapter.parse(input).await
                .map_err(|e| CollectorError::Adapter(e.to_string()))?;

            for event in &events {
                self.record_event(event.clone()).await?;
            }

            Ok(events)
        } else {
            Err(CollectorError::AdapterNotFound(format!("{:?}", agent_type)))
        }
    }

    /// Get all sessions
    pub fn list_sessions(&self, limit: i64) -> Result<Vec<flightdeck_core::SessionSummary>, CollectorError> {
        Ok(self.storage.list_sessions(limit)?)
    }

    /// Get a specific session
    pub fn get_session(&self, session_id: &str) -> Result<Option<flightdeck_core::Session>, CollectorError> {
        Ok(self.storage.get_session(session_id)?)
    }

    /// Get events for a session
    pub fn get_events(&self, session_id: &str) -> Result<Vec<Event>, CollectorError> {
        Ok(self.storage.get_events(session_id)?)
    }

    /// Get metrics
    pub fn get_metrics(&self) -> Result<flightdeck_core::Metrics, CollectorError> {
        Ok(self.storage.get_metrics()?)
    }
}