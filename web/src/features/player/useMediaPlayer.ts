import { useCallback, useEffect, useRef, useState } from 'react'

import type { RefObject } from 'react'

import { api } from '@/lib/api'
import { formatTime } from '@/lib/format'

import { useAppStore } from '@/store/app'

import type { PlaybackTrack } from './playbackTrack'

export interface BufferedRange {
  start: number
  end: number
}

export interface MediaPlayerState {
  paused: boolean
  currentTime: number
  duration: number
  buffered: BufferedRange[]
  volume: number
  muted: boolean
  rate: number
  waiting: boolean
  error: boolean
  pip: boolean
  fullscreen: boolean
  resumed: boolean
}

const initialState: MediaPlayerState = {
  paused: true,
  currentTime: 0,
  duration: 0,
  buffered: [],
  volume: 1,
  muted: false,
  rate: 1,
  waiting: false,
  error: false,
  pip: false,
  fullscreen: false,
  resumed: false,
}

interface PlayerOptions {
  artworkUrl?: string | null
  onEnded?: () => void
}

function readBuffered(media: HTMLMediaElement): BufferedRange[] {
  const ranges: BufferedRange[] = []
  for (let index = 0; index < media.buffered.length; index += 1) {
    ranges.push({ start: media.buffered.start(index), end: media.buffered.end(index) })
  }
  return ranges
}

/**
 * Owns the media element state and persistence for one playback stream.
 *
 * The caller renders the `<video>`/`<audio>` element and passes its ref in;
 * all controls go through the returned actions.
 */
export function useMediaPlayer(
  track: PlaybackTrack | null,
  mediaRef: RefObject<HTMLMediaElement | null>,
  options: PlayerOptions = {},
) {
  const { artworkUrl, onEnded } = options
  const addToast = useAppStore(state => state.addToast)
  const [state, setState] = useState<MediaPlayerState>(initialState)
  const lastSaved = useRef(0)
  const lastTrack = useRef<string | null>(null)
  const resumeApplied = useRef(false)
  const onEndedRef = useRef(onEnded)
  onEndedRef.current = onEnded

  const streamPath = track?.streamPath ?? null
  const url = track?.historyUrl ?? ''
  const label = track?.title ?? ''
  const torrentName = track?.torrentName ?? ''
  const resumePosition = track?.resumePosition ?? null

  const saveProgress = useCallback(
    (force = false) => {
      const media = mediaRef.current
      if (!media || !url || !label) return
      const { currentTime, duration } = media
      if (!Number.isFinite(duration) || duration <= 0 || !Number.isFinite(currentTime)) return
      if (currentTime <= 0) return
      const now = Date.now()
      if (!force && now - lastSaved.current < 5_000) return
      lastSaved.current = now
      void api.saveProgress(url, label, currentTime, duration).catch(() => {
        // Progress is best-effort; a later tick will retry.
      })
    },
    [label, mediaRef, url],
  )

  useEffect(() => {
    const media = mediaRef.current
    if (!media) return

    const sync = () =>
      setState(previous => ({
        ...previous,
        paused: media.paused,
        currentTime: media.currentTime,
        duration: Number.isFinite(media.duration) ? media.duration : previous.duration,
        buffered: readBuffered(media),
        volume: media.volume,
        muted: media.muted,
        rate: media.playbackRate,
      }))

    const onPlay = () => {
      sync()
      setState(previous => ({ ...previous, waiting: false }))
    }
    const onPause = () => {
      sync()
      saveProgress(true)
    }
    const onTimeUpdate = () => {
      sync()
      if ('mediaSession' in navigator && Number.isFinite(media.duration) && media.duration > 0) {
        try {
          navigator.mediaSession.setPositionState({
            duration: media.duration,
            position: Math.min(media.currentTime, media.duration),
            playbackRate: media.playbackRate,
          })
        } catch {
          // Some browsers reject position state without full metadata.
        }
      }
    }
    const onProgress = () => setState(previous => ({ ...previous, buffered: readBuffered(media) }))
    const onWaiting = () => setState(previous => ({ ...previous, waiting: true }))
    const onPlaying = () => setState(previous => ({ ...previous, waiting: false, paused: false }))
    const applyResume = () => {
      if (resumeApplied.current) return
      if (!resumePosition || resumePosition <= 5) return
      if (!Number.isFinite(media.duration) || media.duration <= 0) return
      resumeApplied.current = true
      if (resumePosition >= media.duration - 5) return
      if (Math.abs(media.currentTime - resumePosition) < 2) return
      media.currentTime = resumePosition
      setState(previous => ({ ...previous, resumed: true }))
      addToast('info', `Resuming at ${formatTime(resumePosition)}`)
    }
    const onCanPlay = () => {
      applyResume()
      setState(previous => ({ ...previous, waiting: false, error: false }))
    }
    const onVolume = () =>
      setState(previous => ({ ...previous, volume: media.volume, muted: media.muted }))
    const onRate = () => setState(previous => ({ ...previous, rate: media.playbackRate }))
    const onError = () => setState(previous => ({ ...previous, error: true, waiting: false }))
    const onLoaded = () => {
      sync()
      applyResume()
    }
    const onEnded = () => {
      sync()
      saveProgress(true)
      onEndedRef.current?.()
    }
    const onFullscreen = () =>
      setState(previous => ({ ...previous, fullscreen: Boolean(document.fullscreenElement) }))
    const onEnterPip = () => setState(previous => ({ ...previous, pip: true }))
    const onLeavePip = () => setState(previous => ({ ...previous, pip: false }))

    media.addEventListener('play', onPlay)
    media.addEventListener('pause', onPause)
    media.addEventListener('timeupdate', onTimeUpdate)
    media.addEventListener('loadedmetadata', onLoaded)
    media.addEventListener('loadeddata', onLoaded)
    media.addEventListener('durationchange', sync)
    media.addEventListener('progress', onProgress)
    media.addEventListener('waiting', onWaiting)
    media.addEventListener('playing', onPlaying)
    media.addEventListener('canplay', onCanPlay)
    media.addEventListener('volumechange', onVolume)
    media.addEventListener('ratechange', onRate)
    media.addEventListener('error', onError)
    media.addEventListener('ended', onEnded)
    document.addEventListener('fullscreenchange', onFullscreen)
    media.addEventListener('enterpictureinpicture', onEnterPip)
    media.addEventListener('leavepictureinpicture', onLeavePip)

    sync()
    if (media.readyState >= 1) applyResume()

    return () => {
      saveProgress(true)
      media.removeEventListener('play', onPlay)
      media.removeEventListener('pause', onPause)
      media.removeEventListener('timeupdate', onTimeUpdate)
      media.removeEventListener('loadedmetadata', onLoaded)
      media.removeEventListener('loadeddata', onLoaded)
      media.removeEventListener('durationchange', sync)
      media.removeEventListener('progress', onProgress)
      media.removeEventListener('waiting', onWaiting)
      media.removeEventListener('playing', onPlaying)
      media.removeEventListener('canplay', onCanPlay)
      media.removeEventListener('volumechange', onVolume)
      media.removeEventListener('ratechange', onRate)
      media.removeEventListener('error', onError)
      media.removeEventListener('ended', onEnded)
      document.removeEventListener('fullscreenchange', onFullscreen)
      media.removeEventListener('enterpictureinpicture', onEnterPip)
      media.removeEventListener('leavepictureinpicture', onLeavePip)
    }
  }, [addToast, mediaRef, resumePosition, saveProgress, streamPath, track?.key])

  // Reset per-stream UI state when the stream changes.
  useEffect(() => {
    const key = `${track?.key ?? ''}:${streamPath ?? ''}`
    if (lastTrack.current === key) return
    lastTrack.current = key
    lastSaved.current = 0
    resumeApplied.current = false
    setState({ ...initialState, volume: state.volume, muted: state.muted })
    // Intentionally do not depend on volume; this must only run per stream.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [streamPath, track?.key])

  // Persist every five seconds while playing.
  useEffect(() => {
    if (state.paused) return
    const timer = window.setInterval(() => saveProgress(), 5_000)
    return () => window.clearInterval(timer)
  }, [saveProgress, state.paused])

  useEffect(() => {
    const onVisibility = () => {
      if (document.visibilityState === 'hidden') saveProgress(true)
    }
    document.addEventListener('visibilitychange', onVisibility)
    return () => document.removeEventListener('visibilitychange', onVisibility)
  }, [saveProgress])

  // Media Session metadata and hardware controls.
  useEffect(() => {
    if (!('mediaSession' in navigator) || !label) return
    const media = mediaRef.current
    navigator.mediaSession.metadata = new MediaMetadata({
      title: label,
      artist: torrentName,
      album: 'twatch',
      artwork: artworkUrl ? [{ src: artworkUrl, sizes: '512x512' }] : [],
    })
    const seekTo = (time: number) => {
      if (media) media.currentTime = Math.max(0, time)
    }
    const skip = (delta: number) => {
      if (media) seekTo(media.currentTime + delta)
    }
    try {
      navigator.mediaSession.setActionHandler('play', () => void media?.play())
      navigator.mediaSession.setActionHandler('pause', () => media?.pause())
      navigator.mediaSession.setActionHandler('seekbackward', details =>
        skip(-(details.seekOffset ?? 10)),
      )
      navigator.mediaSession.setActionHandler('seekforward', details =>
        skip(details.seekOffset ?? 10),
      )
      navigator.mediaSession.setActionHandler('seekto', details => {
        if (details.seekTime != null) seekTo(details.seekTime)
      })
    } catch {
      // Older browsers expose a subset of handlers.
    }
    return () => {
      navigator.mediaSession.metadata = null
      for (const action of [
        'play',
        'pause',
        'seekbackward',
        'seekforward',
        'seekto',
      ] as MediaSessionAction[]) {
        try {
          navigator.mediaSession.setActionHandler(action, null)
        } catch {
          // Ignore unsupported actions.
        }
      }
    }
  }, [artworkUrl, label, mediaRef, torrentName])

  const play = useCallback(() => {
    void mediaRef.current?.play().catch(() => {
      setState(previous => ({ ...previous, error: true }))
    })
  }, [mediaRef])

  const pause = useCallback(() => mediaRef.current?.pause(), [mediaRef])

  const toggle = useCallback(() => {
    const media = mediaRef.current
    if (!media) return
    if (media.paused) play()
    else pause()
  }, [mediaRef, pause, play])

  const seek = useCallback(
    (time: number) => {
      const media = mediaRef.current
      if (!media) return
      const duration = Number.isFinite(media.duration) ? media.duration : 0
      media.currentTime = Math.max(0, duration ? Math.min(time, duration) : time)
      setState(previous => ({ ...previous, currentTime: media.currentTime }))
    },
    [mediaRef],
  )

  const skip = useCallback(
    (delta: number) => {
      const media = mediaRef.current
      if (!media) return
      seek(media.currentTime + delta)
    },
    [mediaRef, seek],
  )

  const setVolume = useCallback(
    (volume: number) => {
      const media = mediaRef.current
      if (!media) return
      const next = Math.max(0, Math.min(1, volume))
      media.volume = next
      media.muted = next === 0
    },
    [mediaRef],
  )

  const toggleMute = useCallback(() => {
    const media = mediaRef.current
    if (media) media.muted = !media.muted
  }, [mediaRef])

  const setRate = useCallback(
    (rate: number) => {
      const media = mediaRef.current
      if (!media) return
      media.playbackRate = Math.max(0.25, Math.min(4, rate))
    },
    [mediaRef],
  )

  const toggleFullscreen = useCallback((container: HTMLElement | null) => {
    if (document.fullscreenElement) {
      void document.exitFullscreen()
    } else if (container) {
      void container.requestFullscreen()
    }
  }, [])

  const retry = useCallback(() => {
    setState(previous => ({ ...previous, error: false, waiting: true }))
    const media = mediaRef.current
    if (!media) return
    media.load()
    void media.play().catch(() => {
      // The autoplay policy may require a direct click; the controls stay visible.
    })
  }, [mediaRef])

  const togglePiP = useCallback(async () => {
    const media = mediaRef.current
    if (!media) return
    try {
      if (document.pictureInPictureElement) {
        await document.exitPictureInPicture()
      } else if (document.pictureInPictureEnabled) {
        await (media as HTMLVideoElement).requestPictureInPicture()
      }
    } catch {
      // PiP can be rejected when metadata is not ready.
    }
  }, [mediaRef])

  return {
    state,
    play,
    pause,
    toggle,
    seek,
    skip,
    setVolume,
    toggleMute,
    setRate,
    toggleFullscreen,
    togglePiP,
    retry,
    saveProgress,
  }
}
