import { useRef } from 'react'

import { TriangleAlert } from 'lucide-react'
import type { ReactNode, RefObject } from 'react'

import { cn } from '@/lib/cn'
import { useHlsPlayback } from '@/lib/useHlsPlayback'

export interface VideoSurfaceError {
  title: string
  description: string
  retryLabel?: string
}

interface VideoSurfaceProps {
  source: string
  mediaRef?: RefObject<HTMLMediaElement | null>
  controls?: boolean
  autoPlay?: boolean
  poster?: string
  className?: string
  containerClassName?: string
  onClick?: () => void
  onDoubleClick?: () => void
  reloadOnRetry?: boolean
  errorCopy?: VideoSurfaceError | null
  children?: ReactNode
}

/**
 * One video surface for direct and HLS playback. The player owns controls and
 * subtitles; smaller surfaces such as the local library can use native controls.
 */
export function VideoSurface({
  source,
  mediaRef,
  controls = false,
  autoPlay = false,
  poster,
  className,
  containerClassName,
  onClick,
  onDoubleClick,
  reloadOnRetry = false,
  errorCopy,
  children,
}: VideoSurfaceProps) {
  const localRef = useRef<HTMLVideoElement | null>(null)
  const resolvedRef = mediaRef ?? localRef
  const { attempt, hlsError, isHls, retry } = useHlsPlayback(source, resolvedRef)

  return (
    <div className={cn('relative', containerClassName)}>
      <video
        key={reloadOnRetry ? `${source}:${attempt}` : source}
        ref={resolvedRef as RefObject<HTMLVideoElement | null>}
        src={isHls ? undefined : source}
        controls={controls}
        autoPlay={autoPlay}
        playsInline
        poster={poster}
        className={className}
        onClick={onClick}
        onDoubleClick={onDoubleClick}
      >
        {children}
      </video>

      {hlsError && errorCopy && (
        <div className="absolute inset-0 flex flex-col items-center justify-center gap-2 bg-black/85 px-6 text-center text-white">
          <TriangleAlert className="size-6 text-[var(--warning)]" />
          <p className="text-sm font-semibold">{errorCopy.title}</p>
          <p className="max-w-md text-xs leading-relaxed text-white/60">{errorCopy.description}</p>
          <button
            type="button"
            onClick={retry}
            className="rounded-[2px] border border-white/20 bg-white/10 px-3 py-1 text-xs text-white hover:bg-white/20"
          >
            {errorCopy.retryLabel ?? 'Retry'}
          </button>
        </div>
      )}
    </div>
  )
}
