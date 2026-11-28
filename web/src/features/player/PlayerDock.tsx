import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'

import type { CSSProperties, PointerEvent as ReactPointerEvent } from 'react'

import { clampMiniPosition, defaultMiniPosition, miniWidth } from './miniPlayerPosition'
import type { PlaybackContextValue } from './playbackContext'
import { PlayerStage } from './PlayerStage'

interface SlotRect {
  top: number
  left: number
  width: number
  height: number
}

const MINI_POSITION_KEY = 'twatch.miniplayer-position:v2'

function hasStoredMiniPosition(): boolean {
  try {
    const raw = window.localStorage.getItem(MINI_POSITION_KEY)
    if (!raw) return false
    const parsed = JSON.parse(raw) as { x?: number; y?: number }
    return Number.isFinite(parsed.x) && Number.isFinite(parsed.y)
  } catch {
    return false
  }
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

export function PlayerDock({
  slot,
  playback,
}: {
  slot: HTMLDivElement | null
  playback: PlaybackContextValue
}) {
  const {
    track,
    dockActive,
    mode,
    streamUrl,
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
  } = playback
  const navigate = useNavigate()
  const [rect, setRect] = useState<SlotRect | null>(null)
  const [customPosition, setCustomPosition] = useState(hasStoredMiniPosition)
  const [position, setPosition] = useState(readMiniPosition)
  const positionRef = useRef(position)
  positionRef.current = position
  const dragRef = useRef<{
    pointerId: number
    startX: number
    startY: number
    originX: number
    originY: number
    moved: boolean
  } | null>(null)

  // Keep the default corner in sync with the real viewport. Electron can mount
  // before the window has its final size, and a stale mount-time corner would
  // otherwise leave the mini player at the top instead of bottom-right.
  useEffect(() => {
    if (mode !== 'mini' || customPosition) return
    setPosition(() => {
      const next = defaultMiniPosition()
      positionRef.current = next
      return next
    })
  }, [customPosition, mode])

  useEffect(() => {
    const onResize = () => {
      setPosition(current => {
        const next = customPosition
          ? clampMiniPosition(current.x, current.y)
          : defaultMiniPosition()
        positionRef.current = next
        return next
      })
    }
    window.addEventListener('resize', onResize)
    return () => window.removeEventListener('resize', onResize)
  }, [customPosition])

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
      moved: false,
    }
  }

  const moveDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    const drag = dragRef.current
    if (!drag || drag.pointerId !== event.pointerId) return
    const deltaX = event.clientX - drag.startX
    const deltaY = event.clientY - drag.startY
    if (Math.abs(deltaX) > 3 || Math.abs(deltaY) > 3) drag.moved = true
    const next = clampMiniPosition(drag.originX + deltaX, drag.originY + deltaY)
    positionRef.current = next
    setPosition(next)
  }

  const endDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    const drag = dragRef.current
    if (!drag) return
    dragRef.current = null
    try {
      event.currentTarget.releasePointerCapture(event.pointerId)
    } catch {
      // The pointer may already be released.
    }
    if (!drag.moved) return
    setCustomPosition(true)
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

  if (!track || !dockActive) return null

  const kind = track.media.kind === 'video' ? 'video' : 'audio'
  const playerPath =
    track.source === 'library' && track.library
      ? `/player?library=${encodeURIComponent(track.library.id)}`
      : track.download
        ? `/player?download=${track.download.id}`
        : '/player'
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
        track={track}
        kind={kind}
        streamUrl={streamUrl}
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
        onExpand={() => navigate(playerPath)}
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
