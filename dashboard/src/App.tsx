import { Routes, Route, Link } from 'react-router-dom'
import { Activity, BarChart3, Clock } from 'lucide-react'
import Sessions from './pages/Sessions'
import SessionDetail from './pages/SessionDetail'
import Metrics from './pages/Metrics'

function App() {
  return (
    <div className="min-h-screen bg-gray-50">
      <nav className="bg-white border-b border-gray-200 px-6 py-3">
        <div className="flex items-center justify-between max-w-7xl mx-auto">
          <Link to="/" className="flex items-center gap-2 text-xl font-bold text-gray-900">
            <Activity className="w-6 h-6 text-blue-600" />
            FlightDeck
          </Link>
          <div className="flex items-center gap-6">
            <Link to="/" className="flex items-center gap-1.5 text-gray-600 hover:text-gray-900">
              <Clock className="w-4 h-4" />
              Sessions
            </Link>
            <Link to="/metrics" className="flex items-center gap-1.5 text-gray-600 hover:text-gray-900">
              <BarChart3 className="w-4 h-4" />
              Metrics
            </Link>
          </div>
        </div>
      </nav>
      <main className="max-w-7xl mx-auto px-6 py-8">
        <Routes>
          <Route path="/" element={<Sessions />} />
          <Route path="/session/:id" element={<SessionDetail />} />
          <Route path="/metrics" element={<Metrics />} />
        </Routes>
      </main>
    </div>
  )
}

export default App