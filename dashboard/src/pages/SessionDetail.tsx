import { useParams } from 'react-router-dom'
import { useSession, useEvents } from '../hooks/useApi'
import { useState } from 'react'
import { format } from 'date-fns'
import {
  FileCode, Terminal, AlertCircle,
  FileInput, FileOutput, Trash2, GitBranch,
  CheckCircle, XCircle, Play, Wrench, Brain, MessageSquare
} from 'lucide-react'
import type { Event, EventType } from '../types'

const eventIcons: Record<EventType, React.ReactNode> = {
  SessionStarted: <Play className="w-4 h-4 text-green-500" />,
  SessionEnded: <Play className="w-4 h-4 text-gray-400" />,
  Prompt: <MessageSquare className="w-4 h-4 text-blue-500" />,
  ReadFile: <FileInput className="w-4 h-4 text-cyan-500" />,
  WriteFile: <FileOutput className="w-4 h-4 text-orange-500" />,
  DeleteFile: <Trash2 className="w-4 h-4 text-red-500" />,
  RenameFile: <FileCode className="w-4 h-4 text-yellow-500" />,
  ShellStart: <Terminal className="w-4 h-4 text-purple-500" />,
  ShellEnd: <Terminal className="w-4 h-4 text-gray-400" />,
  ToolCall: <Wrench className="w-4 h-4 text-indigo-500" />,
  ToolResult: <Wrench className="w-4 h-4 text-gray-400" />,
  TestPassed: <CheckCircle className="w-4 h-4 text-green-500" />,
  TestFailed: <XCircle className="w-4 h-4 text-red-500" />,
  Error: <AlertCircle className="w-4 h-4 text-red-500" />,
  Warning: <AlertCircle className="w-4 h-4 text-yellow-500" />,
  GitCommit: <GitBranch className="w-4 h-4 text-green-600" />,
  Notification: <Brain className="w-4 h-4 text-blue-400" />,
}

const eventColors: Record<EventType, string> = {
  SessionStarted: 'border-l-green-500',
  SessionEnded: 'border-l-gray-400',
  Prompt: 'border-l-blue-500',
  ReadFile: 'border-l-cyan-500',
  WriteFile: 'border-l-orange-500',
  DeleteFile: 'border-l-red-500',
  RenameFile: 'border-l-yellow-500',
  ShellStart: 'border-l-purple-500',
  ShellEnd: 'border-l-gray-400',
  ToolCall: 'border-l-indigo-500',
  ToolResult: 'border-l-gray-400',
  TestPassed: 'border-l-green-500',
  TestFailed: 'border-l-red-500',
  Error: 'border-l-red-500',
  Warning: 'border-l-yellow-500',
  GitCommit: 'border-l-green-600',
  Notification: 'border-l-blue-400',
}

function formatDuration(ms: number | null): string {
  if (!ms) return '-'
  const seconds = Math.floor(ms / 1000)
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  const remaining = seconds % 60
  return `${minutes}m ${remaining}s`
}

function truncate(str: string, max: number): string {
  if (str.length <= max) return str
  return str.slice(0, max) + '...'
}

function getEventSummary(event: Event): string {
  const p = event.payload as Record<string, unknown>

  switch (event.event_type) {
    case 'Prompt': {
      const role = p.role as string
      const text = (p.text as string) || ''
      const prefix = role === 'user' ? '👤 User' : '🤖 Assistant'
      return `${prefix}: ${truncate(text, 100)}`
    }
    case 'ReadFile':
      return `📄 ${(p.input as Record<string, string>)?.file_path || 'unknown'}`
    case 'WriteFile':
      return `✏️ ${(p.input as Record<string, string>)?.file_path || 'unknown'}`
    case 'ShellStart':
      return `💻 ${(p.input as Record<string, string>)?.command || 'unknown'}`
    case 'ToolCall':
      return `🔧 ${p.tool as string}: ${truncate(JSON.stringify(p.input), 60)}`
    case 'ToolResult':
      return p.is_error ? `❌ Error` : `✅ Result`
    case 'Notification': {
      const kind = p.kind as string
      if (kind === 'thinking') return `🧠 Thinking: ${truncate(p.content as string, 80)}`
      if (kind === 'usage') return `📊 Tokens: ${p.input_tokens} in / ${p.output_tokens} out`
      return `ℹ️ ${kind}`
    }
    case 'Error':
      return `❌ ${p.error || p.message || 'Unknown error'}`
    default:
      return event.event_type
  }
}

function EventCard({ event, isSelected, onClick }: { event: Event; isSelected: boolean; onClick: () => void }) {
  return (
    <div
      onClick={onClick}
      className={`bg-white rounded border-l-4 ${eventColors[event.event_type]} p-3 cursor-pointer transition-all ${
        isSelected ? 'ring-2 ring-blue-500 shadow-sm' : 'hover:shadow-sm'
      }`}
    >
      <div className="flex items-center gap-2 mb-1">
        {eventIcons[event.event_type]}
        <span className="text-xs text-gray-400">
          {format(new Date(event.timestamp), 'HH:mm:ss')}
        </span>
      </div>
      <p className="text-sm text-gray-700 truncate">{getEventSummary(event)}</p>
    </div>
  )
}

function EventInspector({ event }: { event: Event | null }) {
  if (!event) {
    return (
      <div className="bg-white rounded-lg border border-gray-200 p-6 text-center text-gray-400">
        Select an event to inspect
      </div>
    )
  }

  const p = event.payload as Record<string, unknown>

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-4">
      <h3 className="font-semibold text-gray-900 mb-3">{event.event_type}</h3>
      <div className="space-y-2 text-sm">
        <div>
          <span className="text-gray-500">Time:</span>
          <span className="ml-2">{format(new Date(event.timestamp), 'yyyy-MM-dd HH:mm:ss')}</span>
        </div>
      </div>

      {/* Render content based on event type */}
      <div className="mt-4 space-y-3">
        {event.event_type === 'Prompt' && (
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">
              {(p.role as string) === 'user' ? '👤 User Message' : '🤖 Assistant Response'}
            </h4>
            <div className="bg-gray-50 rounded p-3 text-sm whitespace-pre-wrap max-h-96 overflow-y-auto">
              {p.text as string}
            </div>
            {p.model ? (
              <p className="text-xs text-gray-400 mt-1">Model: {String(p.model)}</p>
            ) : null}
          </div>
        )}

        {event.event_type === 'Notification' && p.kind === 'thinking' && (
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">🧠 Thinking</h4>
            <div className="bg-purple-50 rounded p-3 text-sm whitespace-pre-wrap max-h-96 overflow-y-auto">
              {p.content as string}
            </div>
          </div>
        )}

        {event.event_type === 'Notification' && p.kind === 'usage' && (
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">📊 Token Usage</h4>
            <div className="grid grid-cols-2 gap-2">
              <div className="bg-blue-50 rounded p-2 text-center">
                <p className="text-lg font-bold text-blue-700">{p.input_tokens as number}</p>
                <p className="text-xs text-blue-500">Input</p>
              </div>
              <div className="bg-green-50 rounded p-2 text-center">
                <p className="text-lg font-bold text-green-700">{p.output_tokens as number}</p>
                <p className="text-xs text-green-500">Output</p>
              </div>
            </div>
          </div>
        )}

        {(event.event_type === 'ReadFile' || event.event_type === 'WriteFile') && (
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">📁 File</h4>
            <p className="font-mono text-sm bg-gray-50 rounded p-2">
              {(p.input as Record<string, string>)?.file_path}
            </p>
          </div>
        )}

        {event.event_type === 'ShellStart' && (
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">💻 Command</h4>
            <pre className="font-mono text-sm bg-gray-900 text-green-400 rounded p-3 overflow-x-auto">
              $ {(p.input as Record<string, string>)?.command}
            </pre>
            {(p.input as Record<string, string>)?.description && (
              <p className="text-xs text-gray-400 mt-1">
                {(p.input as Record<string, string>)?.description}
              </p>
            )}
          </div>
        )}

        {event.event_type === 'ToolCall' && (
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">🔧 Tool: {p.tool as string}</h4>
            <pre className="bg-gray-50 rounded p-3 text-xs overflow-auto max-h-64">
              {JSON.stringify(p.input, null, 2)}
            </pre>
          </div>
        )}

        {event.event_type === 'Error' && (
          <div>
            <h4 className="text-sm font-medium text-red-700 mb-2">❌ Error</h4>
            <div className="bg-red-50 rounded p-3 text-sm text-red-800">
              {p.error as string || p.message as string || JSON.stringify(p)}
            </div>
          </div>
        )}

        {/* Raw payload fallback */}
        <details className="mt-4">
          <summary className="text-xs text-gray-400 cursor-pointer hover:text-gray-600">
            Raw Payload
          </summary>
          <pre className="bg-gray-50 rounded p-3 text-xs overflow-auto max-h-64 mt-2">
            {JSON.stringify(event.payload, null, 2)}
          </pre>
        </details>
      </div>
    </div>
  )
}

export default function SessionDetail() {
  const { id } = useParams<{ id: string }>()
  const { data: session, isLoading: sessionLoading } = useSession(id!)
  const { data: events, isLoading: eventsLoading } = useEvents(id!)
  const [selectedEvent, setSelectedEvent] = useState<Event | null>(null)

  if (sessionLoading || eventsLoading) {
    return <div className="text-center py-12 text-gray-500">Loading...</div>
  }

  if (!session) {
    return <div className="text-center py-12 text-red-500">Session not found</div>
  }

  return (
    <div>
      {/* Session Header */}
      <div className="bg-white rounded-lg border border-gray-200 p-6 mb-6">
        <div className="flex items-center justify-between mb-4">
          <h1 className="text-xl font-bold text-gray-900">Session {session.id.slice(0, 8)}</h1>
          <span className="px-3 py-1 bg-gray-100 rounded-full text-sm font-medium">{session.agent_type}</span>
        </div>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div>
            <p className="text-sm text-gray-500">Duration</p>
            <p className="text-lg font-semibold">{formatDuration(session.duration_ms)}</p>
          </div>
          <div>
            <p className="text-sm text-gray-500">Events</p>
            <p className="text-lg font-semibold">{session.event_count}</p>
          </div>
          <div>
            <p className="text-sm text-gray-500">Files Changed</p>
            <p className="text-lg font-semibold">{session.file_count}</p>
          </div>
          <div>
            <p className="text-sm text-gray-500">Failures</p>
            <p className="text-lg font-semibold text-red-600">{session.failure_count}</p>
          </div>
        </div>
        {session.project_path && (
          <p className="text-sm text-gray-400 mt-3 font-mono">{session.project_path}</p>
        )}
      </div>

      {/* Timeline + Inspector */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <h2 className="text-lg font-semibold text-gray-900 mb-3">Timeline</h2>
          <div className="space-y-2 max-h-[600px] overflow-y-auto">
            {events && events.length > 0 ? (
              events.map((event) => (
                <EventCard
                  key={event.id}
                  event={event}
                  isSelected={selectedEvent?.id === event.id}
                  onClick={() => setSelectedEvent(event)}
                />
              ))
            ) : (
              <div className="text-center py-8 text-gray-400">No events recorded</div>
            )}
          </div>
        </div>
        <div>
          <h2 className="text-lg font-semibold text-gray-900 mb-3">Inspector</h2>
          <EventInspector event={selectedEvent} />
        </div>
      </div>
    </div>
  )
}