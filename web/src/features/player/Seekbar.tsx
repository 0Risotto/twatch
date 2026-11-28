import { useEffect, useMemo, useRef, useState } from 'react'

import { Loader2 } from 'lucide-react'
import type { KeyboardEvent as ReactKeyboardEvent, PointerEvent as ReactPointerEvent } from 'react'

import { cn } from '@/lib/cn'
import { formatTime } from '@/lib/format'

import type { BufferedRange } from './useMediaPlayer'

interface SeekbarProps {
  currentTime: number
  duration: number
  buffered: BufferedRange[]
  onSeek: (time: number) => void
  getThumbnail?: (time: number) => Promise<string | null>
  getCachedThumbnail?: (time: number) => string | null
  prefetchThumbnail?: (time: number) => void
}

export function Seekbar({
  currentTime,
  duration,
  buffered,
  onSeek,
  getThumbnail,
  getCachedThumbnail,
  prefetchThumbnail,
}: SeekbarProps) {
  const barRef = useRef<HTMLDivElement>(null)
  const request = useRef(0)
  const [hover, setHover] = useState<number | null>(null)
  const [seeking, setSeeking] = useState(false)
  const [thumbnail, setThumbnail] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  const progress = duration > 0 ? Math.min(100, (currentTime / duration) * 100) : 0
  const display = seeking && hover !== null ? hover : progress
  const hoverTime = hover !== null && duration > 0 ? (hover / 100) * duration : null

  const segments = useMemo(() => {
    if (duration <= 0) return []
    return buffered
      .map(range => ({
        left: Math.max(0, (range.start / duration) * 100),
        width: Math.max(0, ((range.end - range.start) / duration) * 100),
      }))
      .filter(segment => segment.width > 0)
  }, [buffered, duration])

  // Latest-wins: the newest pointer position is requested immediately and
  // stale captures are ignored when they resolve.
  useEffect(() => {
    if (!getThumbnail || hover === null || duration <= 0) {
      setThumbnail(null)
      setLoading(false)
      return
    }
    const time = (hover / 100) * duration
    const cached = getCachedThumbnail?.(time)
    if (cached) {
      setThumbnail(cached)
      setLoading(false)
      return
    }

    const id = ++request.current
    let cancelled = false
    setLoading(true)
    void getThumbnail(time).then(url => {
      if (cancelled || request.current !== id) return
      setThumbnail(url)
      setLoading(false)
    })
    return () => {
      cancelled = true
    }
  }, [duration, getCachedThumbnail, getThumbnail, hover])

  const percentAt = (clientX: number) => {
    const element = barRef.current
    if (!element) return 0
    const rect = element.getBoundingClientRect()
    if (rect.width <= 0) return 0
    return Math.max(0, Math.min(100, ((clientX - rect.left) / rect.width) * 100))
  }

  const seekToPercent = (percent: number) => {
    if (duration <= 0) return
    onSeek((percent / 100) * duration)
  }

  const prefetchAround = (percent: number) => {
    if (!prefetchThumbnail || duration <= 0) return
    const time = (percent / 100) * duration
    prefetchThumbnail(time + 5)
    prefetchThumbnail(time - 5)
  }

  const onPointerDown = (event: ReactPointerEvent<HTMLDivElement>) => {
    event.preventDefault()
    barRef.current?.setPointerCapture(event.pointerId)
    const percent = percentAt(event.clientX)
    setSeeking(true)
    setHover(percent)
    seekToPercent(percent)
  }

  const onPointerMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    const percent = percentAt(event.clientX)
    setHover(percent)
    if (seeking) seekToPercent(percent)
  }

  const stopSeeking = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (seeking) {
      barRef.current?.releasePointerCapture(event.pointerId)
      setSeeking(false)
    }
    const percent = percentAt(event.clientX)
    seekToPercent(percent)
    prefetchAround(percent)
  }

  const onKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    const step = event.shiftKey ? 30 : 5
    if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
      event.preventDefault()
      event.stopPropagation()
      onSeek(currentTime + (event.key === 'ArrowLeft' ? -step : step))
    } else if (event.key === 'Home') {
      event.preventDefault()
      onSeek(0)
    } else if (event.key === 'End') {
      event.preventDefault()
      onSeek(duration)
    }
  }

  return (
    <div
      ref={barRef}
      role="slider"
      tabIndex={0}
      aria-label="Seek"
      aria-valuemin={0}
      aria-valuemax={Math.max(0, Math.round(duration))}
      aria-valuenow={Math.round(currentTime)}
      aria-valuetext={`${formatTime(currentTime)} of ${formatTime(duration)}`}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={stopSeeking}
      onPointerCancel={stopSeeking}
      onPointerLeave={() => {
        if (!seeking) setHover(null)
      }}
      onKeyDown={onKeyDown}
      className="group/seek relative flex h-6 w-full cursor-pointer touch-none items-center outline-none"
    >
      <div className="relative h-[3px] w-full overflow-hidden rounded-[2px] bg-white/20 transition-[height] duration-150 group-hover/seek:h-1.5 group-focus-visible/seek:h-1.5">
        {segments.map((segment, index) => (
          <div
            key={`${segment.left}-${index}`}
            className="absolute inset-y-0 bg-white/25"
            style={{ left: `${segment.left}%`, width: `${segment.width}%` }}
          />
        ))}
        <div
          className="absolute inset-y-0 left-0 bg-[var(--accent)]"
          style={{ width: `${display}%` }}
        />
      </div>

      <div
        className="pointer-events-none absolute top-1/2 h-3.5 w-[3px] -translate-x-1/2 -translate-y-1/2 rounded-[2px] bg-[var(--accent)] shadow-[0_0_0_2px_rgba(0,0,0,0.45)]"
        style={{ left: `${display}%` }}
      />

      {hover !== null && hoverTime !== null && (
        <div
          className="pointer-events-none absolute bottom-full z-20 mb-0.5 flex w-40 -translate-x-1/2 flex-col items-center gap-1.5"
          style={{ left: `clamp(5rem, ${hover}%, calc(100% - 5rem))` }}
        >
          <div className="relative aspect-video w-full overflow-hidden rounded-[4px] border border-white/15 bg-[#07080a] shadow-[0_12px_40px_rgba(0,0,0,0.7)]">
            {thumbnail && (
              <img
                src={thumbnail}
                alt=""
                className={cn(
                  'block h-full w-full object-cover transition-opacity duration-150',
                  loading ? 'opacity-35' : 'opacity-100',
                )}
              />
            )}
            {(loading || !thumbnail) && (
              <div className="absolute inset-0 grid place-items-center">
                {loading ? (
                  <Loader2 className="size-4 animate-spin text-white/70" />
                ) : (
                  <span className="text-[9px] uppercase tracking-[0.16em] text-white/35">
                    no frame
                  </span>
                )}
              </div>
            )}
          </div>
          <div className="rounded-[4px] border border-white/15 bg-black/55 px-2 py-0.5 font-mono text-[10px] tabular text-white/90 shadow-[0_4px_16px_rgba(0,0,0,0.5)] backdrop-blur-md">
            {formatTime(hoverTime)}
          </div>
        </div>
      )}
    </div>
  )
}
