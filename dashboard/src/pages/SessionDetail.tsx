import { useParams } from 'react-router-dom'
import { useSession, useEvents } from '../hooks/useApi'
import { useState } from 'react'
import { format } from 'date-fns'
import {
  Clock, FileCode, Terminal, AlertCircle,
  FileInput, FileOutput, Trash2, GitBranch,
  CheckCircle, XCircle, Play, Wrench
} from 'lucide-react'
import type { Event, EventType } from '../types'

const eventIcons: Record<EventType, React.ReactNode> = {
  SessionStarted: <Play className="w-4 h-4 text-green-500" />,
  SessionEnded: <Play className="w-4 h-4 text-gray-400" />,
  Prompt: <FileInput className="w-4 h-4 text-blue-500" />,
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
  Notification: <Clock className="w-4 h-4 text-blue-400" />,
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

function EventCard({ event, isSelected, onClick }: { event: Event; isSelected: boolean; onClick: () => void }) {
  return (
    <div
      onClick={onClick}
      className={`bg-white rounded border-l-4 ${eventColors[event.event_type]} p-3 cursor-pointer transition-all ${
        isSelected ? 'ring-2 ring-blue-500 shadow-sm' : 'hover:shadow-sm'
      }`}
    >
      <div className="flex items-center gap-2">
        {eventIcons[event.event_type]}
        <span className="text-sm font-medium text-gray-900">{event.event_type}</span>
        <span className="text-xs text-gray-400 ml-auto">
          {format(new Date(event.timestamp), 'HH:mm:ss')}
        </span>
      </div>
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

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-4">
      <h3 className="font-semibold text-gray-900 mb-3">{event.event_type}</h3>
      <div className="space-y-2 text-sm">
        <div>
          <span className="text-gray-500">ID:</span>
          <span className="ml-2 font-mono text-xs">{event.id}</span>
        </div>
        <div>
          <span className="text-gray-500">Time:</span>
          <span className="ml-2">{format(new Date(event.timestamp), 'yyyy-MM-dd HH:mm:ss')}</span>
        </div>
      </div>
      <div className="mt-4">
        <h4 className="text-sm font-medium text-gray-700 mb-2">Payload</h4>
        <pre className="bg-gray-50 rounded p-3 text-xs overflow-auto max-h-96">
          {JSON.stringify(event.payload, null, 2)}
        </pre>
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