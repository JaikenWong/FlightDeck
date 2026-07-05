import { Link } from 'react-router-dom'
import { useSessions, useImportClaudeSessions } from '../hooks/useApi'
import { Bot, Clock, FileCode, AlertCircle, Download } from 'lucide-react'
import type { AgentType } from '../types'

const agentColors: Record<AgentType, string> = {
  claude: 'bg-orange-100 text-orange-700',
  codex: 'bg-green-100 text-green-700',
  gemini: 'bg-blue-100 text-blue-700',
  opencode: 'bg-purple-100 text-purple-700',
  aider: 'bg-yellow-100 text-yellow-700',
  cline: 'bg-pink-100 text-pink-700',
  continue: 'bg-cyan-100 text-cyan-700',
}

function formatDuration(ms: number | null): string {
  if (!ms) return '-'
  const seconds = Math.floor(ms / 1000)
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  const remaining = seconds % 60
  return `${minutes}m ${remaining}s`
}

export default function Sessions() {
  const { data: sessions, isLoading, error } = useSessions()
  const importMutation = useImportClaudeSessions()

  if (isLoading) return <div className="text-center py-12 text-gray-500">Loading sessions...</div>
  if (error) return <div className="text-center py-12 text-red-500">Error: {String(error)}</div>

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-gray-900">Sessions</h1>
        <button
          onClick={() => importMutation.mutate()}
          disabled={importMutation.isPending}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Download className="w-4 h-4" />
          {importMutation.isPending ? 'Importing...' : 'Import Claude Sessions'}
        </button>
      </div>

      {importMutation.isSuccess && (
        <div className="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg text-green-700">
          Imported {importMutation.data.imported} sessions ({importMutation.data.total_found} found)
        </div>
      )}

      {importMutation.isError && (
        <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700">
          Error: {String(importMutation.error)}
        </div>
      )}

      {!sessions || sessions.length === 0 ? (
        <div className="text-center py-12 text-gray-400">
          <Bot className="w-12 h-12 mx-auto mb-3" />
          <p>No sessions recorded yet</p>
          <p className="text-sm mt-2">Click "Import Claude Sessions" to load your history</p>
        </div>
      ) : (
        <div className="space-y-3">
          {sessions.map((session) => (
            <Link
              key={session.id}
              to={`/session/${session.id}`}
              className="block bg-white rounded-lg border border-gray-200 p-4 hover:border-blue-300 hover:shadow-sm transition-all"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <span className={`px-2 py-0.5 rounded text-xs font-medium ${agentColors[session.agent_type]}`}>
                    {session.agent_type}
                  </span>
                  <span className="text-sm text-gray-500 font-mono">{session.id.slice(0, 8)}</span>
                </div>
                <div className="flex items-center gap-4 text-sm text-gray-500">
                  <span className="flex items-center gap-1">
                    <Clock className="w-3.5 h-3.5" />
                    {formatDuration(session.duration_ms)}
                  </span>
                  <span className="flex items-center gap-1">
                    <FileCode className="w-3.5 h-3.5" />
                    {session.event_count} events
                  </span>
                  {session.failure_count > 0 && (
                    <span className="flex items-center gap-1 text-red-500">
                      <AlertCircle className="w-3.5 h-3.5" />
                      {session.failure_count} failures
                    </span>
                  )}
                </div>
              </div>
              {session.project_path && (
                <p className="text-sm text-gray-400 mt-2 font-mono truncate">{session.project_path}</p>
              )}
            </Link>
          ))}
        </div>
      )}
    </div>
  )
}