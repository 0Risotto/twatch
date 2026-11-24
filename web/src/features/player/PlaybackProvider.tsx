import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from 'react'
import type { CSSProperties, PointerEvent as ReactPointerEvent, ReactNode, RefObject } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { api } from '@/lib/api'
import { mediaKind, fileKind, isAudio, isVideo } from '@/lib/fileIcons'
import { useCover } from '@/lib/covers'
import { useAppStore } from '@/store/app'
import type { CoverInfo, DownloadSnapshot } from '@/types/api'
import { PlayerStage } from './PlayerStage'
import { useMediaPlayer } from './useMediaPlayer'
import { usePlayerShortcuts } from './usePlayerShortcuts'
import { useSubtitleTracks } from './useSubtitleTracks'
import type { ExternalSubtitle } from './useSubtitleTracks'
import { useThumbnails } from './useThumbnails'

type MediaPlayer = ReturnType<typeof useMediaPlayer>
type Subtitles = ReturnType<typeof useSubtitleTracks>

interface PlaybackContextValue {
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
  dockActive: boolean
  autoPlay: boolean
  externalSubtitle: ExternalSubtitle | null
  setExternalSubtitle: (subtitle: ExternalSubtitle | null) => void
}

const EXTERNAL_SUBTITLE_PREFIX = 'twatch.external-subtitle:'

function externalSubtitleKey(download: DownloadSnapshot): string {
  return `${EXTERNAL_SUBTITLE_PREFIX}${download.url}|${download.label}`
}

function readExternalSubtitle(download: DownloadSnapshot | null): ExternalSubtitle | null {
  if (!download) return null
  try {
    const raw = window.localStorage.getItem(externalSubtitleKey(download))
    if (!raw) return null
    const parsed = JSON.parse(raw) as { name?: string; content?: string }
    return typeof parsed.name === 'string' && typeof parsed.content === 'string'
      ? { name: parsed.name, content: parsed.content }
      : null
  } catch {
    return null
  }
}

function writeExternalSubtitle(
  download: DownloadSnapshot,
  subtitle: ExternalSubtitle | null,
): boolean {
  try {
    const key = externalSubtitleKey(download)
    if (!subtitle) {
      window.localStorage.removeItem(key)
      return true
    }
    window.localStorage.setItem(key, JSON.stringify(subtitle))
    return true
  } catch {
    return false
  }
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

interface SlotRect {
  top: number
  left: number
  width: number
  height: number
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
  const [externalSubtitle, setExternalSubtitleState] = useState<ExternalSubtitle | null>(null)
  const activeRef = useRef<number | null>(null)
  const restoreId = useRef<number | null>(readActiveDownload())
  const restoreTried = useRef(false)
  const externalKeyRef = useRef<string | null>(null)

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

  useEffect(() => {
    if (!download) {
      externalKeyRef.current = null
      setExternalSubtitleState(null)
      return
    }
    const key = externalSubtitleKey(download)
    if (externalKeyRef.current === key) return
    externalKeyRef.current = key
    setExternalSubtitleState(readExternalSubtitle(download))
  }, [download])

  const cover = useCover(
    download?.torrentName,
    download ? mediaKind(download.torrentName) : undefined,
  )
  const kind = fileKind(download?.fileNames[0] ?? download?.label ?? '')
  const video = isVideo(kind)
  const audio = isAudio(kind)
  const streamUrl = download?.streamPath ?? null
  const mode: 'full' | 'mini' = location.pathname === '/player' ? 'full' : 'mini'

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
    if (candidate?.streamPath) {
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

  const setExternalSubtitle = useCallback(
    (subtitle: ExternalSubtitle | null) => {
      setExternalSubtitleState(subtitle)
      if (!download) return
      if (!writeExternalSubtitle(download, subtitle)) {
        addToast('warning', 'That subtitle is too large to remember; attached for this session')
      }
    },
    [addToast, download],
  )
  const { getThumbnail, getCachedThumbnail, prefetchThumbnail } = useThumbnails(
    video ? streamUrl : null,
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

  const dockActive = Boolean(download && streamUrl && (video || audio))

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
      <PlayerDock slot={slot} />
    </PlaybackContext.Provider>
  )
}

const MINI_WIDTH = 340
const MINI_POSITION_KEY = 'twatch.miniplayer-position'

function miniWidth() {
  return Math.min(MINI_WIDTH, Math.max(200, window.innerWidth - 32))
}

function clampMiniPosition(x: number, y: number) {
  const width = miniWidth()
  const height = (width * 9) / 16
  const maxX = Math.max(8, window.innerWidth - width - 8)
  const maxY = Math.max(8, window.innerHeight - height - 8)
  return {
    x: Math.min(Math.max(8, x), maxX),
    y: Math.min(Math.max(8, y), maxY),
  }
}

function defaultMiniPosition() {
  const width = miniWidth()
  const height = (width * 9) / 16
  const bottom = window.innerWidth < 640 ? 80 : 16
  return clampMiniPosition(window.innerWidth - width - 16, window.innerHeight - height - bottom)
}

function readMiniPosition() {
  try {
    const raw = window.localStorage.getItem(MINI_POSITION_KEY)
    if (!raw) return defaultMiniPosition()
    const parsed = JSON.parse(raw) as { x?: number; y?: number }
    if (Number.isFinite(parsed.x) && Number.isFinite(parsed.y)) {
      return clampMiniPosition(parsed.x as number, parsed.y as number)
    }
  } catch {
    // Fall back to the default corner.
  }
  return defaultMiniPosition()
}

function PlayerDock({ slot }: { slot: HTMLDivElement | null }) {
  const {
    download,
    dockActive,
    mode,
    cover,
    mediaRef,
    containerRef,
    subtitleCanvasRef,
    player,
    subtitles,
    autoPlay,
    hasPrev,
    hasNext,
    playPrev,
    playNext,
    getThumbnail,
    getCachedThumbnail,
    prefetchThumbnail,
    deactivate,
  } = usePlayback()
  const navigate = useNavigate()
  const [rect, setRect] = useState<SlotRect | null>(null)
  const [position, setPosition] = useState(readMiniPosition)
  const positionRef = useRef(position)
  positionRef.current = position
  const dragRef = useRef<{
    pointerId: number
    startX: number
    startY: number
    originX: number
    originY: number
  } | null>(null)

  useEffect(() => {
    const onResize = () => {
      setPosition(current => {
        const next = clampMiniPosition(current.x, current.y)
        positionRef.current = next
        return next
      })
    }
    window.addEventListener('resize', onResize)
    return () => window.removeEventListener('resize', onResize)
  }, [])

  const startDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (mode !== 'mini') return
    event.preventDefault()
    event.currentTarget.setPointerCapture(event.pointerId)
    dragRef.current = {
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      originX: position.x,
      originY: position.y,
    }
  }

  const moveDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    const drag = dragRef.current
    if (!drag || drag.pointerId !== event.pointerId) return
    const next = clampMiniPosition(
      drag.originX + event.clientX - drag.startX,
      drag.originY + event.clientY - drag.startY,
    )
    positionRef.current = next
    setPosition(next)
  }

  const endDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!dragRef.current) return
    dragRef.current = null
    try {
      event.currentTarget.releasePointerCapture(event.pointerId)
    } catch {
      // The pointer may already be released.
    }
    try {
      window.localStorage.setItem(MINI_POSITION_KEY, JSON.stringify(positionRef.current))
    } catch {
      // Position persistence is best-effort.
    }
  }

  useLayoutEffect(() => {
    if (mode !== 'full' || !slot) {
      setRect(null)
      return
    }

    let frame = 0
    const update = () => {
      const bounds = slot.getBoundingClientRect()
      setRect({
        top: bounds.top,
        left: bounds.left,
        width: bounds.width,
        height: bounds.height,
      })
    }

    update()
    const observer = typeof ResizeObserver === 'undefined' ? null : new ResizeObserver(update)
    observer?.observe(slot)
    window.addEventListener('scroll', update, { passive: true })
    window.addEventListener('resize', update)

    // The Shell page transition moves the slot for a few hundred ms.
    const started = performance.now()
    const followTransition = (now: number) => {
      update()
      if (now - started < 700) frame = window.requestAnimationFrame(followTransition)
    }
    frame = window.requestAnimationFrame(followTransition)

    return () => {
      observer?.disconnect()
      window.removeEventListener('scroll', update)
      window.removeEventListener('resize', update)
      window.cancelAnimationFrame(frame)
    }
  }, [mode, slot])

  if (!download || !dockActive) return null

  const kind = fileKind(download.fileNames[0] ?? download.label)
  const style: CSSProperties =
    mode === 'full'
      ? rect && rect.width > 0
        ? {
            position: 'fixed',
            top: rect.top,
            left: rect.left,
            width: rect.width,
            height: rect.height,
            zIndex: 25,
          }
        : {
            position: 'fixed',
            top: 0,
            left: 0,
            width: 1,
            height: 1,
            opacity: 0,
            pointerEvents: 'none',
            zIndex: 25,
          }
      : {
          position: 'fixed',
          left: position.x,
          top: position.y,
          width: miniWidth(),
          zIndex: 30,
        }

  return (
    <div
      className={
        mode === 'full'
          ? 'overflow-hidden bg-black'
          : 'aspect-video max-w-[calc(100vw-2rem)] overflow-hidden rounded-[4px] border border-white/15 bg-black shadow-[0_16px_60px_rgba(0,0,0,0.65)]'
      }
      style={style}
      data-player-dock={mode}
    >
      <PlayerStage
        download={download}
        kind={isVideo(kind) ? 'video' : 'audio'}
        streamUrl={download.streamPath}
        coverUrl={cover?.url}
        mediaRef={mediaRef}
        containerRef={containerRef}
        subtitleCanvasRef={subtitleCanvasRef}
        player={player}
        subtitles={subtitles}
        hasPrev={hasPrev}
        hasNext={hasNext}
        onPrev={playPrev}
        onNext={playNext}
        getThumbnail={getThumbnail}
        getCachedThumbnail={getCachedThumbnail}
        prefetchThumbnail={prefetchThumbnail}
        compact={mode === 'mini'}
        autoPlay={autoPlay}
        onExpand={() => navigate(`/player?download=${download.id}`)}
        onClose={deactivate}
      />
      {mode === 'mini' && (
        <div
          className="absolute inset-x-0 top-0 z-30 flex h-6 cursor-grab touch-none items-start justify-center pt-1.5 active:cursor-grabbing"
          onPointerDown={startDrag}
          onPointerMove={moveDrag}
          onPointerUp={endDrag}
          onPointerCancel={endDrag}
          title="Drag mini player"
        >
          <span className="h-1 w-10 rounded-[1px] bg-white/45 shadow-[0_1px_4px_rgba(0,0,0,0.6)]" />
        </div>
      )}
    </div>
  )
}
