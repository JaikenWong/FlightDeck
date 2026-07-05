import { useMetrics } from '../hooks/useApi'
import { BarChart3, FileCode, Terminal, AlertCircle, Clock, Layers } from 'lucide-react'

function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000)
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  const remaining = seconds % 60
  return `${minutes}m ${remaining}s`
}

export default function Metrics() {
  const { data: metrics, isLoading, error } = useMetrics()

  if (isLoading) return <div className="text-center py-12 text-gray-500">Loading metrics...</div>
  if (error) return <div className="text-center py-12 text-red-500">Error: {String(error)}</div>
  if (!metrics) return <div className="text-center py-12 text-gray-400">No data available</div>

  const cards = [
    { label: 'Total Sessions', value: metrics.total_sessions, icon: Layers, color: 'text-blue-600 bg-blue-50' },
    { label: 'Total Events', value: metrics.total_events, icon: BarChart3, color: 'text-purple-600 bg-purple-50' },
    { label: 'Files Changed', value: metrics.total_files, icon: FileCode, color: 'text-orange-600 bg-orange-50' },
    { label: 'Commands Run', value: metrics.total_commands, icon: Terminal, color: 'text-green-600 bg-green-50' },
    { label: 'Failures', value: metrics.total_failures, icon: AlertCircle, color: 'text-red-600 bg-red-50' },
    { label: 'Avg Duration', value: formatDuration(metrics.avg_duration_ms), icon: Clock, color: 'text-cyan-600 bg-cyan-50' },
  ]

  return (
    <div>
      <h1 className="text-2xl font-bold text-gray-900 mb-6">Metrics</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {cards.map((card) => (
          <div key={card.label} className="bg-white rounded-lg border border-gray-200 p-6">
            <div className="flex items-center gap-3 mb-3">
              <div className={`p-2 rounded-lg ${card.color}`}>
                <card.icon className="w-5 h-5" />
              </div>
              <span className="text-sm text-gray-500">{card.label}</span>
            </div>
            <p className="text-3xl font-bold text-gray-900">{card.value}</p>
          </div>
        ))}
      </div>
    </div>
  )
}