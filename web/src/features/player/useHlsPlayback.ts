import { useCallback, useEffect, useState } from 'react'
import type { RefObject } from 'react'
import Hls from 'hls.js'

/**
 * Attach an HLS source to a media element when the browser cannot play it
 * natively.  Non-HLS sources are left to the element's `src` attribute.
 */
export function useHlsPlayback(
  source: string | null,
  mediaRef: RefObject<HTMLMediaElement | null>,
) {
  const [hlsError, setHlsError] = useState(false)
  const [attempt, setAttempt] = useState(0)
  const isHls = Boolean(source?.endsWith('.m3u8'))

  useEffect(() => {
    setHlsError(false)
    if (!isHls || !source) return
    const media = mediaRef.current
    if (!media) return

    if (!Hls.isSupported()) {
      if (media.canPlayType('application/vnd.apple.mpegurl')) {
        media.src = source
        return () => {
          media.removeAttribute('src')
          media.load()
        }
      }
      setHlsError(true)
      return
    }

    const hls = new Hls({
      enableWorker: true,
      lowLatencyMode: false,
      backBufferLength: 30,
      maxBufferLength: 30,
    })
    hls.loadSource(source)
    hls.attachMedia(media)
    const onError = (_event: string, data: { fatal?: boolean }) => {
      if (data.fatal) setHlsError(true)
    }
    hls.on(Hls.Events.ERROR, onError)
    return () => {
      hls.off(Hls.Events.ERROR, onError)
      hls.destroy()
      media.removeAttribute('src')
      media.load()
    }
  }, [attempt, isHls, mediaRef, source])

  const retry = useCallback(() => setAttempt(value => value + 1), [])

  return { attempt, hlsError, isHls, retry }
}
