import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import type { CSSProperties, PointerEvent as ReactPointerEvent } from 'react'
import { useNavigate } from 'react-router-dom'
import { fileKind, isVideo } from '@/lib/fileIcons'
import { PlayerStage } from './PlayerStage'
import type { PlaybackContextValue } from './PlaybackProvider'

interface SlotRect {
  top: number
  left: number
  width: number
  height: number
}

const MINI_WIDTH = 340
const MINI_POSITION_KEY = 'twatch.miniplayer-position'

function miniWidth() {
  return Math.min(MINI_WIDTH, Math.max(200, window.innerWidth - 32))
}

function clampMiniPosition(x: number, y: number) {
  const width = miniWidth()
  const height = (width * 9) / 16
  const maxX = Math.max(8, window.innerWidth - width - 8)
  const maxY = Math.max(8, window.innerHeight - height - 8)
  return {
    x: Math.min(Math.max(8, x), maxX),
    y: Math.min(Math.max(8, y), maxY),
  }
}

function defaultMiniPosition() {
  const width = miniWidth()
  const height = (width * 9) / 16
  const bottom = window.innerWidth < 640 ? 80 : 16
  return clampMiniPosition(window.innerWidth - width - 16, window.innerHeight - height - bottom)
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
    download,
    dockActive,
    mode,
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
  const [position, setPosition] = useState(readMiniPosition)
  const positionRef = useRef(position)
  positionRef.current = position
  const dragRef = useRef<{
    pointerId: number
    startX: number
    startY: number
    originX: number
    originY: number
  } | null>(null)

  useEffect(() => {
    const onResize = () => {
      setPosition(current => {
        const next = clampMiniPosition(current.x, current.y)
        positionRef.current = next
        return next
      })
    }
    window.addEventListener('resize', onResize)
    return () => window.removeEventListener('resize', onResize)
  }, [])

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
    }
  }

  const moveDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    const drag = dragRef.current
    if (!drag || drag.pointerId !== event.pointerId) return
    const next = clampMiniPosition(
      drag.originX + event.clientX - drag.startX,
      drag.originY + event.clientY - drag.startY,
    )
    positionRef.current = next
    setPosition(next)
  }

  const endDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!dragRef.current) return
    dragRef.current = null
    try {
      event.currentTarget.releasePointerCapture(event.pointerId)
    } catch {
      // The pointer may already be released.
    }
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

  if (!download || !dockActive) return null

  const kind = fileKind(download.fileNames[0] ?? download.label)
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
        download={download}
        kind={isVideo(kind) ? 'video' : 'audio'}
        streamUrl={download.streamPath}
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
        onExpand={() => navigate(`/player?download=${download.id}`)}
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
