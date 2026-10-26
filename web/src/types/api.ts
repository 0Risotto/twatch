export type ThemeMode = 'dark' | 'light'

export interface Palette {
  bg: string
  fg: string
  surface: string
  accent: string
  textPrimary: string
  textDimmed: string
  inputCursor: string
  success: string
  warning: string
  error: string
  folder: string
  gauge: string
  border: string
  hover: string
  badgeDl: string
  badgeStream: string
}

export interface ThemeSpec {
  name: string
  displayName: string
  mode: ThemeMode
}

export interface FileSnapshot {
  index: number
  name: string
  size: number
  selected: boolean
  watched: boolean
  downloaded: boolean
  downloading: boolean
}

export interface SessionSnapshot {
  url: string
  name: string
  files: FileSnapshot[]
  selectedCount: number
  selectedSize: number
  totalSize: number
  downloadRoot: string | null
}

export interface SubtitleTrack {
  index: number
  name: string
}

export type DownloadState =
  | 'starting'
  | 'downloading'
  | 'streaming'
  | 'paused'
  | 'completed'
  | 'failed'

export interface DownloadSnapshot {
  id: number
  torrentId: number | null
  torrentName: string
  label: string
  fileNames: string[]
  url: string
  progress: number
  speed: number
  uploadSpeed: number
  peers: number
  totalSize: number
  downloaded: number
  etaSeconds: number | null
  isStreaming: boolean
  state: DownloadState
  streamPath: string | null
  subtitles: SubtitleTrack[]
  error: string | null
}

export interface PlayerSnapshot {
  running: boolean
  title: string | null
  streamPath: string | null
  downloadId: number | null
}

export interface ConfigView {
  theme: string
  downloadDir: string
  player: string
}

export interface BootstrapSnapshot {
  version: string
  config: ConfigView
  session: SessionSnapshot | null
  downloads: DownloadSnapshot[]
  player: PlayerSnapshot
  themes: ThemeSpec[]
  searchConfigured: boolean
}

export interface HistoryEntry {
  url: string
  customName: string | null
  torrentName: string
  addedAt: number
  watched: boolean
  watchedFiles: string[]
  downloaded: boolean
  downloadedFiles: string[]
}

export interface SearchResult {
  name: string
  infoHash: string
  seeders: number
  leechers: number
  size: number
  username: string
}

export interface FieldMap {
  name: string
  infoHash: string
  seeders: string
  leechers: string
  size: string
  username: string
}

export interface SearchConfig {
  url: string
  headers: Record<string, string>
  fieldMap: FieldMap
}

export interface CoverInfo {
  query: string
  title: string
  url: string | null
  provider: string
  year: number | null
  kind: string
}

export type ToastLevel = 'info' | 'success' | 'warning' | 'error'

export interface ToastItem {
  id: number
  level: ToastLevel
  message: string
}

export type WsFrame =
  | {
      type: 'snapshot'
      revision: number
      session: SessionSnapshot | null
      downloads: DownloadSnapshot[]
      player: PlayerSnapshot
    }
  | { type: 'session'; revision: number; session: SessionSnapshot | null }
  | { type: 'downloads'; revision: number; downloads: DownloadSnapshot[] }
  | { type: 'player'; revision: number; player: PlayerSnapshot }
  | { type: 'toast'; revision: number; level: ToastLevel; message: string }
  | { type: 'resync'; revision: number }
