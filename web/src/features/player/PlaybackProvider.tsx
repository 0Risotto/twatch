import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'

import type { ReactNode } from 'react'

import type { DownloadSnapshot, LibraryEntry } from '@/types/api'

import { api } from '@/lib/api'
import { useCover } from '@/lib/covers'
import { mediaKind } from '@/lib/fileIcons'

import { useAppStore } from '@/store/app'

import type { PlaybackContextValue } from './playbackContext'
import { PlaybackContext } from './playbackContext'
import { downloadTrack, libraryTrack } from './playbackTrack'
import { PlayerDock } from './PlayerDock'
import { useExternalSubtitle } from './useExternalSubtitle'
import { useMediaPlayer } from './useMediaPlayer'
import { usePlayerShortcuts } from './usePlayerShortcuts'
import { useSingleMediaPlayback } from './useSingleMediaPlayback'
import { useSubtitleTracks } from './useSubtitleTracks'
import { useThumbnails } from './useThumbnails'

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

export function PlaybackProvider({ children }: { children: ReactNode }) {
  useSingleMediaPlayback()
  const downloads = useAppStore(state => state.downloads)
  const booted = useAppStore(state => state.booted)
  const addToast = useAppStore(state => state.addToast)
  const location = useLocation()
  const navigate = useNavigate()

  const [activeId, setActiveId] = useState<number | null>(null)
  const [activeLibrary, setActiveLibrary] = useState<LibraryEntry | null>(null)
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

  const track = useMemo(
    () => (activeLibrary ? libraryTrack(activeLibrary) : download ? downloadTrack(download) : null),
    [activeLibrary, download],
  )

  const { externalSubtitle, setExternalSubtitle } = useExternalSubtitle(track, addToast)

  const cover = useCover(
    track?.source === 'download' ? track.torrentName : null,
    track ? mediaKind(track.torrentName) : undefined,
    track?.source === 'download',
  )
  const video = track?.media.kind === 'video'
  const audio = track?.media.kind === 'audio'
  const streamUrl = track?.streamPath ?? null
  const inlineVideo = track?.media.playback === 'direct' && video
  const transcodePath = track?.media.playback === 'hls' && video ? track.streamPath : null
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

  const queue = useMemo(() => track?.queue ?? [], [track?.queue])
  const queueIndex = queue.findIndex(file => file.index === (track?.fileIndex ?? 0))
  const hasPrev = queueIndex > 0
  const hasNext = queueIndex >= 0 && queueIndex + 1 < queue.length

  const activate = useCallback((id: number | null, options?: { restored?: boolean }) => {
    if (id === null) {
      activeRef.current = null
      setActiveId(null)
      setActiveLibrary(null)
      forgetActiveDownload()
      return
    }
    if (!options?.restored) setRestoredPause(false)
    if (activeRef.current !== id) {
      activeRef.current = id
      setFallback(null)
    }
    setActiveLibrary(null)
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

  const activateLibrary = useCallback((entry: LibraryEntry | null) => {
    if (!entry) {
      setActiveLibrary(null)
      return
    }
    activeRef.current = null
    setRestoredPause(false)
    setFallback(null)
    setActiveId(null)
    setActiveLibrary(entry)
    forgetActiveDownload()
  }, [])

  // Restore the last stream after a reload, but keep it paused. If the daemon
  // restarted and the stream is gone, leave the saved download in the list
  // rather than opening an empty miniplayer.
  useEffect(() => {
    if (restoreTried.current || !booted) return
    if (activeId !== null || activeLibrary) {
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
  }, [activate, activeId, activeLibrary, booted, downloads])

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

  const player = useMediaPlayer(track, mediaRef, {
    artworkUrl: cover?.url,
    onEnded: handleEnded,
  })
  const subtitles = useSubtitleTracks(
    track,
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
    enabled: mode === 'full' && Boolean(track && streamUrl && (video || audio)),
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
    setActiveLibrary(null)
    setFallback(null)
    forgetActiveDownload()
  }, [player])

  const registerSlot = useCallback((node: HTMLDivElement | null) => setSlot(node), [])

  const dockActive = Boolean(track && streamUrl && (audio || inlineVideo || transcodePath))

  const value = useMemo<PlaybackContextValue>(
    () => ({
      track,
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
      activateLibrary,
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
      activateLibrary,
      subtitleCanvasRef,
      cover,
      deactivate,
      dockActive,
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
      track,
    ],
  )

  return (
    <PlaybackContext.Provider value={value}>
      {children}
      <PlayerDock slot={slot} playback={value} />
    </PlaybackContext.Provider>
  )
}
