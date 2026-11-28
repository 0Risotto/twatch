import { useCallback, useEffect, useRef, useState } from 'react'

import type { ToastLevel } from '@/types/api'

import type { PlaybackTrack } from './playbackTrack'
import type { ExternalSubtitle } from './useSubtitleTracks'

const EXTERNAL_SUBTITLE_PREFIX = 'twatch.external-subtitle:'

function externalSubtitleKey(track: PlaybackTrack): string {
  const key = track.historyUrl ? `${track.historyUrl}|${track.title}` : track.key
  return `${EXTERNAL_SUBTITLE_PREFIX}${key}`
}

function readExternalSubtitle(track: PlaybackTrack | null): ExternalSubtitle | null {
  if (!track) return null
  try {
    const raw = window.localStorage.getItem(externalSubtitleKey(track))
    if (!raw) return null
    const parsed = JSON.parse(raw) as { name?: string; content?: string }
    return typeof parsed.name === 'string' && typeof parsed.content === 'string'
      ? { name: parsed.name, content: parsed.content }
      : null
  } catch {
    return null
  }
}

function writeExternalSubtitle(track: PlaybackTrack, subtitle: ExternalSubtitle | null): boolean {
  try {
    const key = externalSubtitleKey(track)
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

/**
 * Persists an external subtitle per playback item and keeps it in sync with the
 * active track. Storage failures fall back to session-only subtitles.
 */
export function useExternalSubtitle(
  track: PlaybackTrack | null,
  addToast: (level: ToastLevel, message: string) => number,
) {
  const [externalSubtitle, setExternalSubtitleState] = useState<ExternalSubtitle | null>(null)
  const externalKeyRef = useRef<string | null>(null)

  useEffect(() => {
    if (!track) {
      externalKeyRef.current = null
      setExternalSubtitleState(null)
      return
    }
    const key = externalSubtitleKey(track)
    if (externalKeyRef.current === key) return
    externalKeyRef.current = key
    setExternalSubtitleState(readExternalSubtitle(track))
  }, [track])

  const setExternalSubtitle = useCallback(
    (subtitle: ExternalSubtitle | null) => {
      setExternalSubtitleState(subtitle)
      if (!track) return
      if (!writeExternalSubtitle(track, subtitle)) {
        addToast('warning', 'That subtitle is too large to remember; attached for this session')
      }
    },
    [addToast, track],
  )

  return { externalSubtitle, setExternalSubtitle }
}
