import { useEffect, useRef, useState } from 'react'
import type { RefObject } from 'react'
import { Loader2, Maximize2, Pause, Play, TriangleAlert, X } from 'lucide-react'
import { formatPlaybackTimes } from '@/lib/format'
import { cn } from '@/lib/cn'
import { PlayerControls } from './PlayerControls'
import { useHlsPlayback } from './useHlsPlayback'
import type { useMediaPlayer } from './useMediaPlayer'
import type { useSubtitleTracks } from './useSubtitleTracks'
import type { DownloadSnapshot } from '@/types/api'

type Player = ReturnType<typeof useMediaPlayer>
type Subtitles = ReturnType<typeof useSubtitleTracks>

interface PlayerStageProps {
  download: DownloadSnapshot
  kind: 'video' | 'audio' | 'image'
  streamUrl: string | null
  coverUrl?: string | null
  mediaRef: RefObject<HTMLMediaElement | null>
  containerRef: RefObject<HTMLDivElement | null>
  subtitleCanvasRef: RefObject<HTMLCanvasElement | null>
  player: Player
  subtitles: Subtitles
  hasPrev: boolean
  hasNext: boolean
  onPrev: () => void
  onNext: () => void
  getThumbnail?: (time: number) => Promise<string | null>
  getCachedThumbnail?: (time: number) => string | null
  prefetchThumbnail?: (time: number) => void
  compact?: boolean
  autoPlay?: boolean
  onExpand?: () => void
  onClose?: () => void
}

export function PlayerStage({
  download,
  kind,
  streamUrl,
  coverUrl,
  mediaRef,
  containerRef,
  subtitleCanvasRef,
  player,
  subtitles,
  hasPrev,
  hasNext,
  onPrev,
  onNext,
  getThumbnail,
  getCachedThumbnail,
  prefetchThumbnail,
  compact = false,
  autoPlay = true,
  onExpand,
  onClose,
}: PlayerStageProps) {
  const { state } = player
  const [controlsVisible, setControlsVisible] = useState(true)
  const hideTimer = useRef(0)
  const { hlsError, isHls, retry: retryHls } = useHlsPlayback(streamUrl, mediaRef)
  const { elapsed, remaining } = formatPlaybackTimes(state.currentTime, state.duration)

  const showControls = () => {
    setControlsVisible(true)
    if (compact) return
    window.clearTimeout(hideTimer.current)
    hideTimer.current = window.setTimeout(function hide() {
      if (state.paused) return
      const controls = containerRef.current?.querySelector('.player-controls')
      if (controls?.matches(':hover')) {
        hideTimer.current = window.setTimeout(hide, 1_200)
        return
      }
      setControlsVisible(false)
    }, 2_600)
  }

  useEffect(() => {
    showControls()
    return () => window.clearTimeout(hideTimer.current)
    // Controls stay visible while paused; re-arm whenever playback state changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [state.paused])

  const toggleFullscreen = () => player.toggleFullscreen(containerRef.current)
  const showOverlay = compact || controlsVisible || state.paused

  return (
    <div
      ref={containerRef}
      className={cn(
        'player-stage relative h-full w-full overflow-hidden bg-black',
        !showOverlay && 'cursor-none',
      )}
      onPointerMove={showControls}
      onPointerDown={showControls}
      onTouchStart={showControls}
    >
      {kind === 'video' && streamUrl && !state.error && (
        <video
          key={streamUrl}
          ref={mediaRef as RefObject<HTMLVideoElement | null>}
          src={isHls ? undefined : (streamUrl ?? undefined)}
          poster={coverUrl ?? undefined}
          autoPlay={autoPlay}
          playsInline
          className="h-full w-full bg-black object-contain"
          onClick={() => player.toggle()}
          onDoubleClick={toggleFullscreen}
        >
          {subtitles.nativeTracks.map(track => (
            <track
              key={track.key}
              kind="subtitles"
              src={track.url}
              srcLang={track.srclang}
              label={track.label}
            />
          ))}
        </video>
      )}

      {kind === 'video' && streamUrl && !state.error && (
        <canvas
          ref={subtitleCanvasRef}
          aria-hidden="true"
          className="pointer-events-none absolute inset-0 z-10 h-full w-full"
        />
      )}

      {kind === 'audio' && streamUrl && !state.error && (
        <div
          className={cn(
            'absolute inset-0 flex flex-col items-center justify-center bg-[radial-gradient(ellipse_at_center,color-mix(in_srgb,var(--warning)_14%,transparent),transparent_68%)]',
            compact ? 'gap-2 p-4' : 'gap-6 p-8',
          )}
        >
          {coverUrl ? (
            <img
              src={coverUrl}
              alt=""
              referrerPolicy="no-referrer"
              className={cn(
                'rounded-[var(--radius-lg)] border border-white/10 object-cover shadow-[0_24px_80px_rgba(0,0,0,0.6)]',
                compact ? 'h-20 w-20' : 'h-44 w-44',
              )}
            />
          ) : (
            <div
              className={cn(
                'relative grid place-items-center rounded-full border border-white/15 bg-white/5',
                compact ? 'size-20' : 'size-40',
              )}
            >
              <div className="absolute inset-3 rounded-full border border-white/10" />
              <Play
                className={
                  compact ? 'size-7 text-[var(--warning)]' : 'size-10 text-[var(--warning)]'
                }
              />
            </div>
          )}
          <div className={cn('text-center', compact ? 'max-w-[14rem]' : 'max-w-lg')}>
            <p className={cn('truncate font-semibold text-white', compact ? 'text-xs' : 'text-sm')}>
              {download.label}
            </p>
            <p className={cn('mt-1 truncate text-white/55', compact ? 'text-[10px]' : 'text-xs')}>
              {download.torrentName}
            </p>
          </div>
          <audio
            key={streamUrl}
            ref={mediaRef as RefObject<HTMLAudioElement | null>}
            src={streamUrl}
            autoPlay={autoPlay}
            className="hidden"
          />
        </div>
      )}

      {kind === 'image' && streamUrl && !state.error && (
        <a
          href={streamUrl}
          target="_blank"
          rel="noreferrer"
          className="flex h-full items-center justify-center p-2"
          title="Open original image"
        >
          <img
            src={streamUrl}
            alt={download.label}
            className="max-h-full max-w-full object-contain"
          />
        </a>
      )}

      {state.waiting && !state.error && (
        <div className="pointer-events-none absolute inset-0 grid place-items-center">
          <div className="grid size-16 place-items-center rounded-full bg-black/55 backdrop-blur">
            <Loader2 className="size-7 animate-spin text-white/85" />
          </div>
        </div>
      )}

      {hlsError && !state.error && (
        <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-black/85 px-6 text-center text-white">
          <TriangleAlert className="size-8 text-[var(--warning)]" />
          <p className="text-sm font-semibold">Could not prepare browser playback</p>
          <p className="max-w-md text-xs leading-relaxed text-white/60">
            The local transcoder did not start. Check that ffmpeg is available, then retry.
          </p>
          <button
            type="button"
            onClick={retryHls}
            className="rounded-[2px] border border-white/20 bg-white/10 px-3 py-1 text-xs text-white hover:bg-white/20"
          >
            Retry transcode
          </button>
        </div>
      )}

      {state.error && (
        <div
          className={cn(
            'absolute inset-0 flex flex-col items-center justify-center bg-black/85 text-center text-white',
            compact ? 'gap-2 px-4' : 'gap-3 px-6',
          )}
        >
          <TriangleAlert
            className={compact ? 'size-5 text-[var(--warning)]' : 'size-8 text-[var(--warning)]'}
          />
          <p className={cn('font-semibold', compact ? 'text-xs' : 'text-sm')}>
            {kind === 'audio'
              ? 'Audio codec unsupported'
              : kind === 'image'
                ? 'Could not load this image'
                : 'Container cannot play in the browser'}
          </p>
          {!compact && (
            <p className="max-w-md text-xs leading-relaxed text-white/60">
              The stream is ready. Retry, or download the file and play it locally.
            </p>
          )}
          <div className="mt-1 flex items-center gap-2">
            <button
              type="button"
              onClick={player.retry}
              className="rounded-[2px] border border-white/20 bg-white/10 px-3 py-1 text-xs text-white hover:bg-white/20"
            >
              Retry
            </button>
            {streamUrl && (
              <a
                href={streamUrl}
                target="_blank"
                rel="noreferrer"
                className="rounded-[2px] border border-white/20 px-3 py-1 text-xs text-white/80 hover:bg-white/10"
              >
                Open file
              </a>
            )}
          </div>
        </div>
      )}

      {showOverlay && kind !== 'image' && streamUrl && !state.error && (
        <>
          {!compact && state.paused && (
            <button
              type="button"
              onClick={() => player.toggle()}
              aria-label="Play"
              className="group/play absolute inset-0 z-10 grid place-items-center"
            >
              <Play className="ml-0.5 size-8 fill-current text-white drop-shadow-[0_2px_12px_rgba(0,0,0,0.85)] transition-transform duration-150 group-hover/play:scale-110" />
            </button>
          )}

          {compact ? (
            <div className="absolute inset-x-0 bottom-0 z-20 bg-gradient-to-t from-black/95 via-black/70 to-transparent px-2.5 pb-2 pt-8">
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={player.toggle}
                  aria-label={state.paused ? 'Play' : 'Pause'}
                  className="grid size-8 shrink-0 place-items-center text-white/90 hover:text-white"
                >
                  {state.paused ? (
                    <Play className="ml-0.5 size-4 fill-current" />
                  ) : (
                    <Pause className="size-4 fill-current" />
                  )}
                </button>
                <div className="min-w-0 flex-1">
                  <p className="truncate text-[11px] font-semibold text-white">{download.label}</p>
                  <div className="mt-1 flex items-center gap-2">
                    <div className="h-[2px] flex-1 overflow-hidden rounded-[1px] bg-white/20">
                      <div
                        className="h-full bg-[var(--accent)]"
                        style={{
                          width: `${state.duration > 0 ? Math.min(100, (state.currentTime / state.duration) * 100) : 0}%`,
                        }}
                      />
                    </div>
                    <span className="shrink-0 font-mono text-[9px] tabular text-white/60">
                      {elapsed} / {remaining}
                    </span>
                  </div>
                </div>
                {onExpand && (
                  <button
                    type="button"
                    onClick={onExpand}
                    aria-label="Expand player"
                    title="Expand player"
                    className="grid size-7 shrink-0 place-items-center rounded-[2px] text-white/75 hover:bg-white/10 hover:text-white"
                  >
                    <Maximize2 className="size-3.5" />
                  </button>
                )}
                {onClose && (
                  <button
                    type="button"
                    onClick={onClose}
                    aria-label="Close player"
                    title="Close player"
                    className="grid size-7 shrink-0 place-items-center rounded-[2px] text-white/75 hover:bg-white/10 hover:text-white"
                  >
                    <X className="size-4" />
                  </button>
                )}
              </div>
            </div>
          ) : (
            <div
              className={cn(
                'player-controls transition-opacity duration-200',
                showOverlay ? 'opacity-100' : 'pointer-events-none opacity-0',
              )}
            >
              <PlayerControls
                state={state}
                buffered={state.buffered}
                hasPrev={hasPrev}
                hasNext={hasNext}
                tracks={subtitles.tracks}
                selectedSubtitle={subtitles.selected}
                subtitleDelay={subtitles.delay}
                subtitleSize={subtitles.size}
                getThumbnail={getThumbnail}
                getCachedThumbnail={getCachedThumbnail}
                prefetchThumbnail={prefetchThumbnail}
                onToggle={player.toggle}
                onSeek={player.seek}
                onPrev={onPrev}
                onNext={onNext}
                onVolume={player.setVolume}
                onToggleMute={player.toggleMute}
                onRate={player.setRate}
                onToggleFullscreen={toggleFullscreen}
                onTogglePip={() => void player.togglePiP()}
                onSelectSubtitle={subtitles.selectTrack}
                onSubtitleDelay={subtitles.adjustDelay}
                onResetSubtitleDelay={subtitles.resetDelay}
                onSubtitleSize={subtitles.setSize}
              />
            </div>
          )}
        </>
      )}
    </div>
  )
}
