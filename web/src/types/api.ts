import type { DownloadSnapshot } from './generated/DownloadSnapshot'
import type { PlayerSnapshot } from './generated/PlayerSnapshot'
import type { SessionSnapshot } from './generated/SessionSnapshot'
import type { ToastLevel } from './generated/ToastLevel'

export type { BootstrapSnapshot } from './generated/BootstrapSnapshot'
export type { ConfigView } from './generated/ConfigView'
export type { Cover as CoverInfo } from './generated/Cover'
export type { DownloadSnapshot } from './generated/DownloadSnapshot'
export type { DownloadState } from './generated/DownloadState'
export type { FieldMapDto as FieldMap } from './generated/FieldMapDto'
export type { FileSnapshot } from './generated/FileSnapshot'
export type { HistoryEntryDto as HistoryEntry } from './generated/HistoryEntryDto'
export type { PlaybackProgress } from './generated/PlaybackProgress'
export type { PlayerSnapshot } from './generated/PlayerSnapshot'
export type { QueueFile } from './generated/QueueFile'
export type { SearchConfigDto as SearchConfig } from './generated/SearchConfigDto'
export type { SearchResult } from './generated/SearchResult'
export type { SessionSnapshot } from './generated/SessionSnapshot'
export type { SubtitleTrack } from './generated/SubtitleTrack'
export type { ThemeMode } from './generated/ThemeMode'
export type { ThemeSpec } from './generated/ThemeSpec'
export type { ToastLevel } from './generated/ToastLevel'

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
