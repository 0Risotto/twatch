import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type { RefObject } from 'react'
import JASSUB from 'jassub'
import workerUrl from 'jassub/dist/jassub-worker.js?url'
import wasmUrl from 'jassub/dist/jassub-worker.wasm?url'
import legacyWasmUrl from 'jassub/dist/jassub-worker.wasm.js?url'
import modernWasmUrl from 'jassub/dist/jassub-worker-modern.wasm?url'
import defaultFontUrl from 'jassub/dist/default.woff2?url'
import {
  matchingSubtitles,
  rawSubtitleUrl,
  subtitleExtension,
  subtitleLabel,
  subtitleLanguage,
  subtitleToVtt,
  subtitleUrl,
} from '@/lib/subtitles'
import type { DownloadSnapshot, SubtitleTrack } from '@/types/api'

export type SubtitleSize = 'sm' | 'md' | 'lg'

export interface ExternalSubtitle {
  name: string
  content: string
}

export interface SubtitleSource extends SubtitleTrack {
  external?: boolean
  content?: string
}

export interface NativeSubtitleTrack {
  key: string
  name: string
  url: string
  srclang: string
  label: string
}

const JASSUB_ASSET_VERSION = 'jassub-1'
const assetUrl = (url: string) => `${url}?v=${JASSUB_ASSET_VERSION}`

interface SubtitlePreference {
  name: string | null
  delay: number
  size: SubtitleSize
}

const STORAGE_KEY = 'twatch.subtitle-preferences'
const SIZES: SubtitleSize[] = ['sm', 'md', 'lg']

function readStore(): Record<string, SubtitlePreference> {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) return {}
    const parsed = JSON.parse(raw) as Record<string, SubtitlePreference>
    return parsed && typeof parsed === 'object' ? parsed : {}
  } catch {
    return {}
  }
}

function writeStore(store: Record<string, SubtitlePreference>): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(store))
  } catch {
    // Preference persistence is best-effort.
  }
}

function defaultTrack(fileName: string, tracks: SubtitleTrack[]): string | null {
  const matching = matchingSubtitles(fileName, tracks)
  const english = matching.find(track => subtitleLanguage(track.name).code.startsWith('en'))
  return (english ?? matching[0])?.name ?? tracks[0]?.name ?? null
}

/**
 * Manages subtitle track selection, delay, and styling.
 *
 * Delay is applied directly to the loaded VTT cues so it works with the
 * browser's native `<track>` renderer without a second subtitle engine.
 */
export function useSubtitleTracks(
  download: DownloadSnapshot | null,
  mediaRef: RefObject<HTMLMediaElement | null>,
  containerRef: RefObject<HTMLElement | null>,
  subtitleCanvasRef: RefObject<HTMLCanvasElement | null>,
  externalSubtitle: ExternalSubtitle | null,
) {
  const torrentName = download?.torrentName ?? ''
  const torrentId = download?.torrentId ?? null
  const fileName = download?.fileNames[0] ?? download?.label ?? ''
  const streamPath = download?.streamPath ?? null
  const baseTracks = download?.subtitles ?? []
  const tracks = useMemo<SubtitleSource[]>(() => {
    if (!externalSubtitle) return baseTracks
    return [
      ...baseTracks,
      {
        index: -1,
        name: externalSubtitle.name,
        external: true,
        content: externalSubtitle.content,
      },
    ]
  }, [baseTracks, externalSubtitle])
  const trackKey = `${baseTracks.map(track => `${track.index}:${track.name}`).join('|')}|ext:${
    externalSubtitle?.name ?? ''
  }:${externalSubtitle?.content.length ?? 0}`

  const [selected, setSelected] = useState<string | null>(null)
  const [delay, setDelay] = useState(0)
  const [size, setSize] = useState<SubtitleSize>('md')
  const [assFailed, setAssFailed] = useState(false)
  const baseTimes = useRef(new WeakMap<TextTrackCue, { start: number; end: number }>())
  const tracksRef = useRef(tracks)
  tracksRef.current = tracks
  const jassubRef = useRef<JASSUB | null>(null)

  const selectedTrack = tracks.find(track => track.name === selected) ?? null
  const selectedIndex = selectedTrack?.index ?? null
  const selectedExtension = selectedTrack ? subtitleExtension(selectedTrack.name) : ''
  const selectedIsAss = selectedExtension === 'ass' || selectedExtension === 'ssa'
  const selectedExternalContent = selectedTrack?.external ? (selectedTrack.content ?? '') : ''
  const nativeFallback = selectedIsAss && assFailed

  const nativeTracks = useMemo<NativeSubtitleTrack[]>(
    () =>
      tracks.map(track => {
        const language = subtitleLanguage(track.name)
        const label = subtitleLabel(track)
        return {
          key: `${track.external ? 'external' : 'torrent'}:${track.index}:${track.name}`,
          name: track.name,
          url: track.external
            ? `data:text/vtt;charset=utf-8,${encodeURIComponent(
                subtitleToVtt(track.name, track.content ?? ''),
              )}`
            : torrentId !== null
              ? subtitleUrl(torrentId, track.index)
              : '',
          srclang: language.code,
          label: track.external ? `${label} (external)` : label,
        }
      }),
    [externalSubtitle?.content, torrentId, trackKey],
  )

  // Give a newly selected/loaded track a fresh chance to initialize JASSUB.
  useEffect(() => {
    setAssFailed(false)
  }, [selected, selectedExtension, selectedExternalContent, streamPath, torrentId])

  // Restore the last choice for this torrent.
  useEffect(() => {
    if (!torrentName || !trackKey) {
      setSelected(null)
      return
    }
    const current = tracksRef.current
    const saved = readStore()[torrentName]
    if (saved) {
      setDelay(saved.delay)
      setSize(SIZES.includes(saved.size) ? saved.size : 'md')
      const exists = saved.name === null || current.some(track => track.name === saved.name)
      setSelected(exists ? saved.name : defaultTrack(fileName, current))
      return
    }
    setDelay(0)
    setSize('md')
    setSelected(defaultTrack(fileName, current))
  }, [fileName, torrentName, trackKey])

  // Loading an external file should select it immediately, after restore.
  useEffect(() => {
    if (externalSubtitle) setSelected(externalSubtitle.name)
  }, [externalSubtitle])

  const applyDelay = useCallback(() => {
    const media = mediaRef.current
    if (!media) return
    for (let index = 0; index < media.textTracks.length; index += 1) {
      const cues = media.textTracks[index].cues
      if (!cues) continue
      for (let cueIndex = 0; cueIndex < cues.length; cueIndex += 1) {
        const cue = cues[cueIndex]
        let base = baseTimes.current.get(cue)
        if (!base) {
          base = { start: cue.startTime, end: cue.endTime }
          baseTimes.current.set(cue, base)
        }
        const start = Math.max(0, base.start + delay)
        const end = Math.max(start + 0.05, base.end + delay)
        cue.startTime = start
        cue.endTime = end
      }
    }
  }, [delay, mediaRef])

  const syncNativeModes = useCallback(() => {
    const media = mediaRef.current
    if (!media) return
    const current = tracksRef.current
    for (let index = 0; index < media.textTracks.length; index += 1) {
      const track = current[index]
      const visible = track?.name === selected && (!selectedIsAss || nativeFallback)
      media.textTracks[index].mode = visible ? 'showing' : 'disabled'
    }
    applyDelay()
  }, [applyDelay, mediaRef, nativeFallback, selected, selectedIsAss])

  // Show the selected native track unless JASSUB owns ASS/SSA rendering.
  useEffect(() => {
    syncNativeModes()
    const timer = window.setTimeout(syncNativeModes, 250)
    return () => window.clearTimeout(timer)
  }, [syncNativeModes, streamPath, torrentId, trackKey])

  // ASS/SSA tracks are rendered by libass through JASSUB for real styling.
  useEffect(() => {
    const video = mediaRef.current as HTMLVideoElement | null
    const canvas = subtitleCanvasRef.current
    if (selectedIndex === null || !selectedIsAss || !video || !canvas || assFailed) {
      return
    }
    if (!selectedExternalContent && torrentId === null) return

    let instance: JASSUB | null = null
    let disposed = false
    const fail = () => {
      if (disposed) return
      setAssFailed(true)
      try {
        instance?.destroy()
      } catch {
        // Ignore teardown failures.
      }
      if (jassubRef.current === instance) jassubRef.current = null
    }

    try {
      instance = new JASSUB({
        video,
        canvas,
        ...(selectedExternalContent
          ? { subContent: selectedExternalContent }
          : { subUrl: rawSubtitleUrl(torrentId!, selectedIndex) }),
        workerUrl: assetUrl(workerUrl),
        wasmUrl: assetUrl(wasmUrl),
        legacyWasmUrl: assetUrl(legacyWasmUrl),
        modernWasmUrl: assetUrl(modernWasmUrl),
        timeOffset: 0,
        availableFonts: {
          'liberation sans': assetUrl(defaultFontUrl),
          arial: assetUrl(defaultFontUrl),
          roboto: assetUrl(defaultFontUrl),
        },
        fallbackFont: 'liberation sans',
      })
      jassubRef.current = instance
      instance.addEventListener('error', fail)
    } catch {
      fail()
      return
    }

    const resize = () => {
      try {
        instance?.resize()
      } catch {
        // The canvas may not have a video frame yet.
      }
    }
    resize()
    const retryTimers = [
      window.setTimeout(resize, 250),
      window.setTimeout(resize, 1_000),
    ]
    const observer = typeof ResizeObserver === 'undefined' ? null : new ResizeObserver(resize)
    observer?.observe(canvas)
    video.addEventListener('loadedmetadata', resize)
    video.addEventListener('loadeddata', resize)
    video.addEventListener('canplay', resize)
    video.addEventListener('playing', resize)
    const onReady = () => resize()
    instance.addEventListener('ready', onReady)

    return () => {
      disposed = true
      observer?.disconnect()
      for (const timer of retryTimers) window.clearTimeout(timer)
      video.removeEventListener('loadedmetadata', resize)
      video.removeEventListener('loadeddata', resize)
      video.removeEventListener('canplay', resize)
      video.removeEventListener('playing', resize)
      instance?.removeEventListener('ready', onReady)
      instance?.removeEventListener('error', fail)
      try {
        instance?.destroy()
      } catch {
        // Ignore teardown failures.
      }
      if (jassubRef.current === instance) jassubRef.current = null
    }
  }, [
    subtitleCanvasRef,
    assFailed,
    mediaRef,
    selectedExternalContent,
    selectedIsAss,
    selectedIndex,
    streamPath,
    torrentId,
  ])

  // JASSUB reads its offset property on every render tick.
  useEffect(() => {
    const instance = jassubRef.current
    const video = mediaRef.current
    if (!instance || !video) return
    ;(instance as JASSUB & { timeOffset: number }).timeOffset = delay
    try {
      instance.setCurrentTime(video.paused, video.currentTime + delay, video.playbackRate)
    } catch {
      // The worker may still be initializing.
    }
  }, [delay, mediaRef, selectedIndex, selectedIsAss, streamPath])

  // Cues arrive after metadata; listen for track load events as they land.
  useEffect(() => {
    const media = mediaRef.current
    if (!media) return
    const onLoaded = () => {
      syncNativeModes()
      applyDelay()
    }
    media.addEventListener('loadedmetadata', onLoaded)
    media.addEventListener('canplay', onLoaded)
    const elements = [...media.querySelectorAll('track')]
    for (const element of elements) {
      element.addEventListener('load', onLoaded)
    }
    const timer = window.setTimeout(onLoaded, 500)
    return () => {
      media.removeEventListener('loadedmetadata', onLoaded)
      media.removeEventListener('canplay', onLoaded)
      for (const element of elements) {
        element.removeEventListener('load', onLoaded)
      }
      window.clearTimeout(timer)
    }
  }, [applyDelay, mediaRef, streamPath, syncNativeModes, torrentId, trackKey])

  // Apply subtitle size through a CSS variable.
  useEffect(() => {
    const container = containerRef.current
    if (!container) return
    const pixels = size === 'sm' ? 16 : size === 'lg' ? 26 : 21
    container.style.setProperty('--subtitle-size', `${pixels}px`)
  }, [containerRef, size, streamPath])

  // Persist the preference for this torrent.
  useEffect(() => {
    if (!torrentName || !trackKey) return
    const store = readStore()
    store[torrentName] = { name: selected, delay, size }
    writeStore(store)
  }, [delay, selected, size, torrentName, trackKey])

  const selectTrack = useCallback((name: string | null) => setSelected(name), [])
  const adjustDelay = useCallback(
    (delta: number) =>
      setDelay(previous => {
        const next = Math.round((previous + delta) * 10) / 10
        return Math.max(-60, Math.min(60, next))
      }),
    [],
  )
  const resetDelay = useCallback(() => setDelay(0), [])

  return {
    tracks,
    nativeTracks,
    selected,
    selectTrack,
    delay,
    setDelay,
    adjustDelay,
    resetDelay,
    size,
    setSize,
    selectedIsAss,
  }
}
