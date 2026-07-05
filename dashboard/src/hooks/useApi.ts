import { useQuery } from '@tanstack/react-query'
import type { ApiResponse, Session, SessionSummary, Event, Metrics } from '../types'

const API_BASE = '/api'

async function fetchApi<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`)
  if (!res.ok) throw new Error(`API error: ${res.status}`)
  const json: ApiResponse<T> = await res.json()
  if (!json.success) throw new Error(json.error || 'Unknown error')
  return json.data as T
}

export function useSessions() {
  return useQuery({
    queryKey: ['sessions'],
    queryFn: () => fetchApi<SessionSummary[]>('/sessions'),
  })
}

export function useSession(id: string) {
  return useQuery({
    queryKey: ['session', id],
    queryFn: () => fetchApi<Session>(`/sessions/${id}`),
    enabled: !!id,
  })
}

export function useEvents(sessionId: string) {
  return useQuery({
    queryKey: ['events', sessionId],
    queryFn: () => fetchApi<Event[]>(`/sessions/${sessionId}/events`),
    enabled: !!sessionId,
  })
}

export function useMetrics() {
  return useQuery({
    queryKey: ['metrics'],
    queryFn: () => fetchApi<Metrics>('/metrics'),
  })
}