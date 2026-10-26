import type {
  BootstrapSnapshot,
  ConfigView,
  CoverInfo,
  DownloadSnapshot,
  HistoryEntry,
  PlayerSnapshot,
  SearchConfig,
  SearchResult,
  SessionSnapshot,
  ThemeSpec,
} from '@/types/api'

export class ApiError extends Error {
  status: number

  constructor(message: string, status: number) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(path, {
    credentials: 'same-origin',
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...(options.headers ?? {}),
    },
  })
  if (!response.ok) {
    let message = response.statusText
    try {
      const body = (await response.json()) as { error?: { message?: string } }
      message = body.error?.message ?? message
    } catch {
      // Non-JSON error body.
    }
    throw new ApiError(message || `Request failed (${response.status})`, response.status)
  }
  if (response.status === 204) return undefined as T
  return (await response.json()) as T
}

function post<T>(path: string, body?: unknown): Promise<T> {
  return request<T>(path, { method: 'POST', body: body === undefined ? undefined : JSON.stringify(body) })
}

function patch<T>(path: string, body: unknown): Promise<T> {
  return request<T>(path, { method: 'PATCH', body: JSON.stringify(body) })
}

function put<T>(path: string, body: unknown): Promise<T> {
  return request<T>(path, { method: 'PUT', body: JSON.stringify(body) })
}

function del<T>(path: string, body?: unknown): Promise<T> {
  return request<T>(path, {
    method: 'DELETE',
    body: body === undefined ? undefined : JSON.stringify(body),
  })
}

export const api = {
  bootstrap: () => request<BootstrapSnapshot>('/api/bootstrap'),
  health: () => request<{ ok: boolean; version: string }>('/api/health'),

  preview: (url: string) => post<SessionSnapshot>('/api/session/preview', { url }),
  closeSession: () => post<{ ok: boolean }>('/api/session/close'),
  selectFiles: (indices: number[], selected: boolean) =>
    post<SessionSnapshot>('/api/session/files/select', { indices, selected }),
  watchFile: (index: number) => post<DownloadSnapshot>('/api/session/files/watch', { index }),
  downloadFiles: (indices: number[]) =>
    post<DownloadSnapshot>('/api/session/files/download', { indices }),
  renameFile: (index: number, name: string) =>
    patch<SessionSnapshot>('/api/session/files/rename', { index, name }),
  deleteFiles: (indices: number[]) =>
    del<{ deleted: number; failed: string[] }>('/api/session/files', { indices }),

  downloads: () => request<DownloadSnapshot[]>('/api/downloads'),
  pauseDownload: (id: number) => post<{ ok: boolean }>(`/api/downloads/${id}/pause`),
  resumeDownload: (id: number) => post<{ ok: boolean }>(`/api/downloads/${id}/resume`),
  removeDownload: (id: number) => del<{ ok: boolean }>(`/api/downloads/${id}`),

  search: (q: string, page = 0) =>
    request<SearchResult[]>(`/api/search?q=${encodeURIComponent(q)}&page=${page}`),
  getSearchConfig: () => request<SearchConfig>('/api/search/config'),
  putSearchConfig: (config: SearchConfig) => put<SearchConfig>('/api/search/config', config),

  history: () => request<HistoryEntry[]>('/api/history'),
  renameHistory: (url: string, name: string) =>
    patch<{ ok: boolean }>('/api/history', { url, name }),
  deleteHistory: (url: string) => del<{ ok: boolean }>('/api/history', { url }),
  openHistory: (url: string) => post<SessionSnapshot>('/api/history/open', { url }),

  player: () => request<PlayerSnapshot>('/api/player'),
  openPlayer: (downloadId: number) =>
    post<PlayerSnapshot>('/api/player/open', { downloadId }),
  stopPlayer: () => post<{ ok: boolean }>('/api/player/stop'),

  covers: (items: Array<{ title: string; kind?: string }>) =>
    post<CoverInfo[]>('/api/covers', { items }),

  themes: () => request<ThemeSpec[]>('/api/themes'),
  config: () => request<ConfigView>('/api/config'),
  patchConfig: (body: { theme?: string; player?: string }) =>
    patch<ConfigView>('/api/config', body),
}
