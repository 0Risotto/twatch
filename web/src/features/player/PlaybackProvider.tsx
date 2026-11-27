import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react'
import type { ReactNode, RefObject } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { api } from '@/lib/api'
import { mediaKind } from '@/lib/fileIcons'
import { useCover } from '@/lib/covers'
import { useAppStore } from '@/store/app'
import type { CoverInfo, DownloadSnapshot } from '@/types/api'
import { PlayerDock } from './PlayerDock'
import { useMediaPlayer } from './useMediaPlayer'
import { usePlayerShortcuts } from './usePlayerShortcuts'
import { useExternalSubtitle } from './useExternalSubtitle'
import { useSubtitleTracks } from './useSubtitleTracks'
import type { ExternalSubtitle } from './useSubtitleTracks'
import { useThumbnails } from './useThumbnails'

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

const ACTIVE_DOWNLOAD_KEY = 'twatch.active-download'

function readActiveDownload(): number | null {
  try {
    const raw = window.localStorage.getItem(ACTIVE_DOWNLOAD_KEY)
    const id = Number(raw)
    return Number.isFinite(id) && id > 0 ? id : null
  } catch {
    return null
  }
}

function rememberActiveDownload(id: number): void {
  try {
    window.localStorage.setItem(ACTIVE_DOWNLOAD_KEY, String(id))
  } catch {
    // Best-effort persistence.
  }
}

function forgetActiveDownload(): void {
  try {
    window.localStorage.removeItem(ACTIVE_DOWNLOAD_KEY)
  } catch {
    // Best-effort persistence.
  }
}

const PlaybackContext = createContext<PlaybackContextValue | null>(null)

export function usePlayback(): PlaybackContextValue {
  const context = useContext(PlaybackContext)
  if (!context) throw new Error('usePlayback must be used inside PlaybackProvider')
  return context
}

export function PlaybackProvider({ children }: { children: ReactNode }) {
  const downloads = useAppStore(state => state.downloads)
  const booted = useAppStore(state => state.booted)
  const addToast = useAppStore(state => state.addToast)
  const location = useLocation()
  const navigate = useNavigate()

  const [activeId, setActiveId] = useState<number | null>(null)
  const [fallback, setFallback] = useState<DownloadSnapshot | null>(null)
  const [slot, setSlot] = useState<HTMLDivElement | null>(null)
  const [restoredPause, setRestoredPause] = useState(false)
  const activeRef = useRef<number | null>(null)
  const restoreId = useRef<number | null>(readActiveDownload())
  const restoreTried = useRef(false)

  const mediaRef = useRef<HTMLMediaElement | null>(null)
  const containerRef = useRef<HTMLDivElement | null>(null)
  const subtitleCanvasRef = useRef<HTMLCanvasElement | null>(null)

  const storeDownload = activeId
    ? (downloads.find(download => download.id === activeId) ?? null)
    : null
  const download = storeDownload ?? (fallback && fallback.id === activeId ? fallback : null)

  useEffect(() => {
    if (storeDownload && fallback?.id === activeId) setFallback(null)
  }, [activeId, fallback, storeDownload])

  const { externalSubtitle, setExternalSubtitle } = useExternalSubtitle(download, addToast)

  const cover = useCover(
    download?.torrentName,
    download ? mediaKind(download.torrentName) : undefined,
  )
  const video = download?.media?.kind === 'video'
  const audio = download?.media?.kind === 'audio'
  const streamUrl = download?.media?.streamPath ?? null
  const inlineVideo = download?.media?.playback === 'direct' && video
  const transcodePath =
    download?.media?.playback === 'hls' && video ? download.media.streamPath : null
  const mode: 'full' | 'mini' = location.pathname === '/player' ? 'full' : 'mini'

  useEffect(() => {
    const id = download?.id
    if (!id || download?.media?.streamPath) return
    let cancelled = false

    const run = async () => {
      let resumeRequested = false
      try {
        const deadline = Date.now() + 30_000
        while (!cancelled && Date.now() < deadline) {
          await new Promise(resolve => setTimeout(resolve, 1_000))
          if (cancelled) return
          const list = await api.downloads()
          useAppStore.getState().setDownloads(list)
          const entry = list.find(item => item.id === id)
          if (entry?.media?.streamPath || entry?.state === 'failed') return
          if (!resumeRequested && (entry?.state === 'paused' || entry?.state === 'completed')) {
            await api.resumeDownload(id)
            resumeRequested = true
          }
        }
        if (!cancelled) addToast('warning', 'Stream is taking too long to prepare')
      } catch (error) {
        if (!cancelled) {
          addToast('error', error instanceof Error ? error.message : 'Could not prepare stream')
        }
      }
    }

    void run()
    return () => {
      cancelled = true
    }
  }, [addToast, download?.id, download?.media?.streamPath])

  const queue = useMemo(() => download?.queue ?? [], [download?.queue])
  const queueIndex = queue.findIndex(file => file.index === (download?.fileIndex ?? 0))
  const hasPrev = queueIndex > 0
  const hasNext = queueIndex >= 0 && queueIndex + 1 < queue.length

  const activate = useCallback((id: number | null, options?: { restored?: boolean }) => {
    if (id === null) {
      activeRef.current = null
      setActiveId(null)
      forgetActiveDownload()
      return
    }
    if (!options?.restored) setRestoredPause(false)
    if (activeRef.current !== id) {
      activeRef.current = id
      setFallback(null)
    }
    rememberActiveDownload(id)
    setActiveId(id)
    const state = useAppStore.getState()
    if (!state.downloads.some(item => item.id === id)) {
      void api
        .downloads()
        .then(list => useAppStore.getState().setDownloads(list))
        .catch(() => {
          // The next WebSocket snapshot will fill this in.
        })
    }
  }, [])

  // Restore the last stream after a reload, but keep it paused. If the daemon
  // restarted and the stream is gone, leave the saved download in the list
  // rather than opening an empty miniplayer.
  useEffect(() => {
    if (restoreTried.current || !booted) return
    if (activeId !== null) {
      restoreTried.current = true
      return
    }
    restoreTried.current = true
    const id = restoreId.current
    if (!id) return
    const candidate = downloads.find(item => item.id === id)
    if (candidate?.media?.streamPath) {
      setRestoredPause(true)
      activate(id, { restored: true })
    } else {
      forgetActiveDownload()
    }
  }, [activate, activeId, booted, downloads])

  const switchTo = useCallback(
    async (index: number) => {
      if (!download) return
      try {
        const next = await api.watchFile(index)
        activeRef.current = next.id
        setRestoredPause(false)
        rememberActiveDownload(next.id)
        setFallback(next)
        setActiveId(next.id)
        if (location.pathname === '/player') {
          navigate(`/player?download=${next.id}`, { replace: true, state: location.state })
        }
      } catch (reason) {
        addToast('error', reason instanceof Error ? reason.message : 'Could not start playback')
      }
    },
    [addToast, download, location.pathname, location.state, navigate],
  )

  const playPrev = useCallback(() => {
    if (hasPrev) void switchTo(queue[queueIndex - 1].index)
  }, [hasPrev, queue, queueIndex, switchTo])

  const playNext = useCallback(() => {
    if (hasNext) void switchTo(queue[queueIndex + 1].index)
  }, [hasNext, queue, queueIndex, switchTo])

  const handleEnded = useCallback(() => {
    if (hasNext) playNext()
  }, [hasNext, playNext])

  const player = useMediaPlayer(download, mediaRef, {
    artworkUrl: cover?.url,
    onEnded: handleEnded,
  })
  const subtitles = useSubtitleTracks(
    download,
    mediaRef,
    containerRef,
    subtitleCanvasRef,
    externalSubtitle,
  )

  const { getThumbnail, getCachedThumbnail, prefetchThumbnail } = useThumbnails(
    video && !transcodePath ? streamUrl : null,
  )

  const cycleSubtitles = useCallback(() => {
    const tracks = subtitles.tracks
    if (tracks.length === 0) return
    if (!subtitles.selected) {
      subtitles.selectTrack(tracks[0].name)
      return
    }
    const index = tracks.findIndex(track => track.name === subtitles.selected)
    subtitles.selectTrack(index >= 0 && index + 1 < tracks.length ? tracks[index + 1].name : null)
  }, [subtitles])

  usePlayerShortcuts({
    enabled: mode === 'full' && Boolean(download && streamUrl && (video || audio)),
    toggle: player.toggle,
    seek: player.seek,
    skip: player.skip,
    setVolume: player.setVolume,
    volume: () => mediaRef.current?.volume ?? 1,
    toggleMute: player.toggleMute,
    toggleFullscreen: () => player.toggleFullscreen(containerRef.current),
    togglePip: () => void player.togglePiP(),
    cycleSubtitles,
    next: playNext,
    prev: playPrev,
    setRate: player.setRate,
    rate: () => mediaRef.current?.playbackRate ?? 1,
  })

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') return
      if (document.fullscreenElement) void document.exitFullscreen()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])

  const deactivate = useCallback(() => {
    player.saveProgress(true)
    mediaRef.current?.pause()
    activeRef.current = null
    setRestoredPause(false)
    setActiveId(null)
    setFallback(null)
    forgetActiveDownload()
  }, [player])

  const registerSlot = useCallback((node: HTMLDivElement | null) => setSlot(node), [])

  const dockActive = Boolean(download && streamUrl && (audio || inlineVideo || transcodePath))

  const value = useMemo<PlaybackContextValue>(
    () => ({
      download,
      cover,
      player,
      subtitles,
      mediaRef,
      containerRef,
      subtitleCanvasRef,
      getThumbnail,
      getCachedThumbnail,
      prefetchThumbnail,
      hasPrev,
      hasNext,
      playPrev,
      playNext,
      switchTo,
      activate,
      deactivate,
      registerSlot,
      mode,
      streamUrl,
      dockActive,
      autoPlay: !restoredPause,
      externalSubtitle,
      setExternalSubtitle,
    }),
    [
      activate,
      subtitleCanvasRef,
      cover,
      deactivate,
      dockActive,
      download,
      externalSubtitle,
      getCachedThumbnail,
      getThumbnail,
      hasNext,
      hasPrev,
      mediaRef,
      mode,
      streamUrl,
      playNext,
      playPrev,
      player,
      prefetchThumbnail,
      registerSlot,
      restoredPause,
      setExternalSubtitle,
      subtitles,
      switchTo,
    ],
  )

  return (
    <PlaybackContext.Provider value={value}>
      {children}
      <PlayerDock slot={slot} playback={value} />
    </PlaybackContext.Provider>
  )
}
