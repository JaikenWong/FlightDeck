export type AgentType = 'claude' | 'codex' | 'gemini' | 'opencode' | 'aider' | 'cline' | 'continue'

export type EventType =
  | 'SessionStarted'
  | 'SessionEnded'
  | 'Prompt'
  | 'ReadFile'
  | 'WriteFile'
  | 'DeleteFile'
  | 'RenameFile'
  | 'ShellStart'
  | 'ShellEnd'
  | 'ToolCall'
  | 'ToolResult'
  | 'TestPassed'
  | 'TestFailed'
  | 'Error'
  | 'Warning'
  | 'GitCommit'
  | 'Notification'

export interface Event {
  id: string
  session_id: string
  timestamp: string
  event_type: EventType
  payload: Record<string, unknown>
}

export interface Session {
  id: string
  agent_type: AgentType
  project_path: string | null
  model: string | null
  branch: string | null
  started_at: string
  ended_at: string | null
  duration_ms: number | null
  event_count: number
  file_count: number
  command_count: number
  failure_count: number
}

export interface SessionSummary {
  id: string
  agent_type: AgentType
  project_path: string | null
  model: string | null
  started_at: string
  duration_ms: number | null
  event_count: number
  failure_count: number
}

export interface Metrics {
  total_sessions: number
  total_events: number
  total_files: number
  total_commands: number
  total_failures: number
  avg_duration_ms: number
}

export interface ApiResponse<T> {
  success: boolean
  data: T | null
  error: string | null
}