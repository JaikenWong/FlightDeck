use rusqlite::{Connection, params, OptionalExtension};
use flightdeck_core::{Event, EventType, Session, SessionSummary, Metrics, AgentType};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, debug};
use std::sync::Mutex;

use crate::error::StorageError;

pub struct Storage {
    conn: Mutex<Connection>,
}

impl Storage {
    pub fn new(db_path: &str) -> Result<Self, StorageError> {
        let conn = Connection::open(db_path)?;
        let storage = Self { conn: Mutex::new(conn) };
        storage.init_tables()?;
        Ok(storage)
    }

    fn init_tables(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Parse(e.to_string()))?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                agent_type TEXT NOT NULL,
                project_path TEXT,
                model TEXT,
                branch TEXT,
                started_at DATETIME NOT NULL,
                ended_at DATETIME,
                duration_ms INTEGER,
                event_count INTEGER DEFAULT 0,
                file_count INTEGER DEFAULT 0,
                command_count INTEGER DEFAULT 0,
                failure_count INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                timestamp DATETIME NOT NULL,
                event_type TEXT NOT NULL,
                payload JSON NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            );

            CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id);
            CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
            CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
            "
        )?;
        info!("Database tables initialized");
        Ok(())
    }

    pub fn create_session(&self, session: &Session) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Parse(e.to_string()))?;
        conn.execute(
            "INSERT INTO sessions (id, agent_type, project_path, model, branch, started_at, ended_at, duration_ms, event_count, file_count, command_count, failure_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                session.id,
                session.agent_type.to_string(),
                session.project_path,
                session.model,
                session.branch,
                session.started_at.to_rfc3339(),
                session.ended_at.map(|dt| dt.to_rfc3339()),
                session.duration_ms,
                session.event_count,
                session.file_count,
                session.command_count,
                session.failure_count,
            ],
        )?;
        debug!("Created session: {}", session.id);
        Ok(())
    }

    pub fn update_session(&self, session: &Session) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Parse(e.to_string()))?;
        conn.execute(
            "UPDATE sessions SET 
                ended_at = ?1,
                duration_ms = ?2,
                event_count = ?3,
                file_count = ?4,
                command_count = ?5,
                failure_count = ?6
             WHERE id = ?7",
            params![
                session.ended_at.map(|dt| dt.to_rfc3339()),
                session.duration_ms,
                session.event_count,
                session.file_count,
                session.command_count,
                session.failure_count,
                session.id,
            ],
        )?;
        debug!("Updated session: {}", session.id);
        Ok(())
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<Session>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Parse(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, agent_type, project_path, model, branch, started_at, ended_at, duration_ms, event_count, file_count, command_count, failure_count
             FROM sessions WHERE id = ?1"
        )?;

        let session = stmt.query_row(params![session_id], |row| {
            Ok(Session {
                id: row.get(0)?,
                agent_type: row.get::<_, String>(1)?.parse().unwrap_or(AgentType::Claude),
                project_path: row.get(2)?,
                model: row.get(3)?,
                branch: row.get(4)?,
                started_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?
                    .with_timezone(&Utc),
                ended_at: row.get::<_, Option<String>>(6)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                duration_ms: row.get(7)?,
                event_count: row.get(8)?,
                file_count: row.get(9)?,
                command_count: row.get(10)?,
                failure_count: row.get(11)?,
            })
        }).optional()?;

        Ok(session)
    }

    pub fn list_sessions(&self, limit: i64) -> Result<Vec<SessionSummary>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Parse(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, agent_type, project_path, model, started_at, duration_ms, event_count, failure_count
             FROM sessions ORDER BY started_at DESC LIMIT ?1"
        )?;

        let sessions = stmt.query_map(params![limit], |row| {
            Ok(SessionSummary {
                id: row.get(0)?,
                agent_type: row.get::<_, String>(1)?.parse().unwrap_or(AgentType::Claude),
                project_path: row.get(2)?,
                model: row.get(3)?,
                started_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?
                    .with_timezone(&Utc),
                duration_ms: row.get(5)?,
                event_count: row.get(6)?,
                failure_count: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    pub fn insert_event(&self, event: &Event) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Parse(e.to_string()))?;
        conn.execute(
            "INSERT INTO events (id, session_id, timestamp, event_type, payload)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                event.id.to_string(),
                event.session_id,
                event.timestamp.to_rfc3339(),
                serde_json::to_string(&event.event_type)?,
                serde_json::to_string(&event.payload)?,
            ],
        )?;
        debug!("Inserted event: {} for session: {}", event.id, event.session_id);
        Ok(())
    }

    pub fn get_events(&self, session_id: &str) -> Result<Vec<Event>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Parse(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, timestamp, event_type, payload
             FROM events WHERE session_id = ?1 ORDER BY timestamp"
        )?;

        let events = stmt.query_map(params![session_id], |row| {
            let event_type_str: String = row.get(3)?;
            let event_type: EventType = serde_json::from_str(&event_type_str)
                .or_else(|_| serde_json::from_str(&format!("\"{}\"", event_type_str)))
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
            
            let payload_str: String = row.get(4)?;
            let payload: serde_json::Value = serde_json::from_str(&payload_str)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

            Ok(Event {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_else(|_| Uuid::new_v4()),
                session_id: row.get(1)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?
                    .with_timezone(&Utc),
                event_type,
                payload,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(events)
    }

    pub fn get_events_by_type(&self, session_id: &str, event_type: &EventType) -> Result<Vec<Event>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Parse(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, timestamp, event_type, payload
             FROM events WHERE session_id = ?1 AND event_type = ?2 ORDER BY timestamp"
        )?;

        let event_type_str = serde_json::to_string(event_type)?;
        let events = stmt.query_map(params![session_id, event_type_str], |row| {
            let event_type_str: String = row.get(3)?;
            let event_type: EventType = serde_json::from_str(&event_type_str)
                .or_else(|_| serde_json::from_str(&format!("\"{}\"", event_type_str)))
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
            
            let payload_str: String = row.get(4)?;
            let payload: serde_json::Value = serde_json::from_str(&payload_str)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

            Ok(Event {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_else(|_| Uuid::new_v4()),
                session_id: row.get(1)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?
                    .with_timezone(&Utc),
                event_type,
                payload,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(events)
    }

    pub fn get_metrics(&self) -> Result<Metrics, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Parse(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT 
                COUNT(*) as total_sessions,
                COALESCE(SUM(event_count), 0) as total_events,
                COALESCE(SUM(file_count), 0) as total_files,
                COALESCE(SUM(command_count), 0) as total_commands,
                COALESCE(SUM(failure_count), 0) as total_failures,
                COALESCE(AVG(duration_ms), 0) as avg_duration
             FROM sessions"
        )?;

        let metrics = stmt.query_row([], |row| {
            Ok(Metrics {
                total_sessions: row.get(0)?,
                total_events: row.get(1)?,
                total_files: row.get(2)?,
                total_commands: row.get(3)?,
                total_failures: row.get(4)?,
                avg_duration_ms: row.get(5)?,
            })
        })?;

        Ok(metrics)
    }
}