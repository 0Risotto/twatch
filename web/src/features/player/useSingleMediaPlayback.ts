import { useEffect } from 'react'

/** Pause every media element except the one that just started playing. */
export function pauseOtherMedia(target: EventTarget | null): void {
  if (typeof document === 'undefined' || !(target instanceof HTMLMediaElement)) return
  for (const media of document.querySelectorAll<HTMLMediaElement>('video,audio')) {
    if (media !== target) media.pause()
  }
}

/**
 * Keep one audio/video element playing at a time. The torrent player and the
 * local library player share the same document, so starting one must stop the
 * other instead of mixing two soundtracks.
 */
export function useSingleMediaPlayback(): void {
  useEffect(() => {
    const onPlaying = (event: Event) => pauseOtherMedia(event.target)
    document.addEventListener('playing', onPlaying, true)
    return () => document.removeEventListener('playing', onPlaying, true)
  }, [])
}
