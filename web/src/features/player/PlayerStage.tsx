import { useEffect, useRef, useState } from 'react'
import type { RefObject } from 'react'
import { Loader2, Play, TriangleAlert } from 'lucide-react'
import { subtitleLabel, subtitleLanguage, subtitleUrl } from '@/lib/subtitles'
import { cn } from '@/lib/cn'
import { PlayerControls } from './PlayerControls'
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
  player: Player
  subtitles: Subtitles
  hasPrev: boolean
  hasNext: boolean
  onPrev: () => void
  onNext: () => void
  getThumbnail?: (time: number) => Promise<string | null>
  getCachedThumbnail?: (time: number) => string | null
  prefetchThumbnail?: (time: number) => void
}

export function PlayerStage({
  download,
  kind,
  streamUrl,
  coverUrl,
  mediaRef,
  containerRef,
  player,
  subtitles,
  hasPrev,
  hasNext,
  onPrev,
  onNext,
  getThumbnail,
  getCachedThumbnail,
  prefetchThumbnail,
}: PlayerStageProps) {
  const { state } = player
  const [controlsVisible, setControlsVisible] = useState(true)
  const hideTimer = useRef(0)

  const showControls = () => {
    setControlsVisible(true)
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
  const showOverlay = controlsVisible || state.paused

  return (
    <div
      ref={containerRef}
      className={cn(
        'player-stage relative aspect-video w-full overflow-hidden bg-black',
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
          src={streamUrl}
          poster={coverUrl ?? undefined}
          autoPlay
          playsInline
          className="h-full w-full bg-black object-contain"
          onClick={() => player.toggle()}
          onDoubleClick={toggleFullscreen}
        >
          {download.subtitles.map(track => (
            <track
              key={track.index}
              kind="subtitles"
              src={subtitleUrl(download.torrentId!, track.index)}
              srcLang={subtitleLanguage(track.name).code}
              label={subtitleLabel(track)}
              default={track.name === subtitles.selected}
            />
          ))}
        </video>
      )}

      {kind === 'audio' && streamUrl && !state.error && (
        <div className="absolute inset-0 flex flex-col items-center justify-center gap-6 bg-[radial-gradient(ellipse_at_center,color-mix(in_srgb,var(--warning)_14%,transparent),transparent_68%)] p-8">
          {coverUrl ? (
            <img
              src={coverUrl}
              alt=""
              referrerPolicy="no-referrer"
              className="h-44 w-44 rounded-[var(--radius-lg)] border border-white/10 object-cover shadow-[0_24px_80px_rgba(0,0,0,0.6)]"
            />
          ) : (
            <div className="relative grid size-40 place-items-center rounded-full border border-white/15 bg-white/5">
              <div className="absolute inset-3 rounded-full border border-white/10" />
              <Play className="size-10 text-[var(--warning)]" />
            </div>
          )}
          <div className="max-w-lg text-center">
            <p className="truncate text-sm font-semibold text-white">{download.label}</p>
            <p className="mt-1 truncate text-xs text-white/55">{download.torrentName}</p>
          </div>
          <audio
            key={streamUrl}
            ref={mediaRef as RefObject<HTMLAudioElement | null>}
            src={streamUrl}
            autoPlay
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
          <img src={streamUrl} alt={download.label} className="max-h-full max-w-full object-contain" />
        </a>
      )}

      {state.waiting && !state.error && (
        <div className="pointer-events-none absolute inset-0 grid place-items-center">
          <div className="grid size-16 place-items-center rounded-full bg-black/55 backdrop-blur">
            <Loader2 className="size-7 animate-spin text-white/85" />
          </div>
        </div>
      )}

      {state.error && (
        <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-black/80 px-6 text-center text-white">
          <TriangleAlert className="size-8 text-[var(--warning)]" />
          <p className="text-sm font-semibold">
            {kind === 'audio'
              ? 'This audio codec cannot play in the browser'
              : kind === 'image'
                ? 'Could not load this image'
                : 'This video container cannot play in the browser'}
          </p>
          <p className="max-w-md text-xs leading-relaxed text-white/60">
            The stream is ready. Hand it to mpv or VLC, or download the file and play it locally.
          </p>
        </div>
      )}

      {showOverlay && kind !== 'image' && streamUrl && !state.error && (
        <>
          {state.paused && (
            <button
              type="button"
              onClick={() => player.toggle()}
              aria-label="Play"
              className="group/play absolute inset-0 z-10 grid place-items-center"
            >
              <Play className="ml-0.5 size-8 fill-current text-white drop-shadow-[0_2px_12px_rgba(0,0,0,0.85)] transition-transform duration-150 group-hover/play:scale-110" />
            </button>
          )}
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
        </>
      )}
    </div>
  )
}
