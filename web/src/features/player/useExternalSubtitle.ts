import { useCallback, useEffect, useRef, useState } from 'react'
import type { ExternalSubtitle } from './useSubtitleTracks'
import type { DownloadSnapshot, ToastLevel } from '@/types/api'

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

/**
 * Persists an external subtitle per torrent file and keeps it in sync with the
 * active download.  Storage failures fall back to session-only subtitles.
 */
export function useExternalSubtitle(
  download: DownloadSnapshot | null,
  addToast: (level: ToastLevel, message: string) => number,
) {
  const [externalSubtitle, setExternalSubtitleState] = useState<ExternalSubtitle | null>(null)
  const externalKeyRef = useRef<string | null>(null)

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

  return { externalSubtitle, setExternalSubtitle }
}
