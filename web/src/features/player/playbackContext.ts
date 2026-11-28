import { createContext, useContext } from 'react'

import type { RefObject } from 'react'

import type { CoverInfo, DownloadSnapshot } from '@/types/api'

import type { useMediaPlayer } from './useMediaPlayer'
import type { ExternalSubtitle, useSubtitleTracks } from './useSubtitleTracks'

type MediaPlayer = ReturnType<typeof useMediaPlayer>
type Subtitles = ReturnType<typeof useSubtitleTracks>

export interface PlaybackContextValue {
  download: DownloadSnapshot | null
  cover: CoverInfo | null
  player: MediaPlayer
  subtitles: Subtitles
  mediaRef: RefObject<HTMLMediaElement | null>
  containerRef: RefObject<HTMLDivElement | null>
  subtitleCanvasRef: RefObject<HTMLCanvasElement | null>
  getThumbnail: (time: number) => Promise<string | null>
  getCachedThumbnail: (time: number) => string | null
  prefetchThumbnail: (time: number) => void
  hasPrev: boolean
  hasNext: boolean
  playPrev: () => void
  playNext: () => void
  switchTo: (index: number) => void
  activate: (id: number | null) => void
  deactivate: () => void
  registerSlot: (node: HTMLDivElement | null) => void
  mode: 'full' | 'mini'
  streamUrl: string | null
  dockActive: boolean
  autoPlay: boolean
  externalSubtitle: ExternalSubtitle | null
  setExternalSubtitle: (subtitle: ExternalSubtitle | null) => void
}

export const PlaybackContext = createContext<PlaybackContextValue | null>(null)

export function usePlayback(): PlaybackContextValue {
  const context = useContext(PlaybackContext)
  if (!context) throw new Error('usePlayback must be used inside PlaybackProvider')
  return context
}
