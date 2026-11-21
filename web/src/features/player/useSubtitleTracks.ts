import { useCallback, useEffect, useRef, useState } from 'react'
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
  subtitleLanguage,
} from '@/lib/subtitles'
import type { DownloadSnapshot, SubtitleTrack } from '@/types/api'

export type SubtitleSize = 'sm' | 'md' | 'lg'

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
) {
  const torrentName = download?.torrentName ?? ''
  const torrentId = download?.torrentId ?? null
  const fileName = download?.fileNames[0] ?? download?.label ?? ''
  const streamPath = download?.streamPath ?? null
  const tracks = download?.subtitles ?? []
  const trackKey = tracks.map(track => `${track.index}:${track.name}`).join('|')

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
  const nativeFallback = selectedIsAss && assFailed

  // Give a newly selected/loaded track a fresh chance to initialize JASSUB.
  useEffect(() => {
    setAssFailed(false)
  }, [selected, streamPath, torrentId])

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

  // Show the selected native track unless JASSUB owns ASS/SSA rendering.
  useEffect(() => {
    const media = mediaRef.current
    if (!media) return
    const current = tracksRef.current
    for (let index = 0; index < media.textTracks.length; index += 1) {
      const track = current[index]
      const visible = track?.name === selected && (!selectedIsAss || nativeFallback)
      media.textTracks[index].mode = visible ? 'showing' : 'disabled'
    }
    applyDelay()
  }, [
    applyDelay,
    mediaRef,
    nativeFallback,
    selected,
    selectedIsAss,
    streamPath,
    torrentId,
    trackKey,
  ])

  // ASS/SSA tracks are rendered by libass through JASSUB for real styling.
  useEffect(() => {
    const video = mediaRef.current as HTMLVideoElement | null
    const canvas = subtitleCanvasRef.current
    if (selectedIndex === null || !selectedIsAss || !video || !canvas || !torrentId || assFailed) {
      return
    }

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
        subUrl: rawSubtitleUrl(torrentId, selectedIndex),
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
    const observer = typeof ResizeObserver === 'undefined' ? null : new ResizeObserver(resize)
    observer?.observe(canvas)
    video.addEventListener('loadedmetadata', resize)
    const onReady = () => resize()
    instance.addEventListener('ready', onReady)

    return () => {
      disposed = true
      observer?.disconnect()
      video.removeEventListener('loadedmetadata', resize)
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
    const onLoaded = () => applyDelay()
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
  }, [applyDelay, mediaRef, streamPath, torrentId, trackKey])

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
