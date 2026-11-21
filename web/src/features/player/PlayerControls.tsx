import { useEffect, useRef, useState } from 'react'
import {
  Captions,
  CaptionsOff,
  Gauge,
  Maximize,
  Minimize,
  Pause,
  PictureInPicture2,
  Play,
  Settings2,
  SkipBack,
  SkipForward,
  Volume1,
  Volume2,
  VolumeX,
} from 'lucide-react'
import { formatPlaybackTimes, formatTime } from '@/lib/format'
import { subtitleLabel } from '@/lib/subtitles'
import { cn } from '@/lib/cn'
import { Seekbar } from './Seekbar'
import type { SubtitleSize } from './useSubtitleTracks'
import type { BufferedRange, MediaPlayerState } from './useMediaPlayer'
import type { SubtitleTrack } from '@/types/api'

const SPEEDS = [0.5, 0.75, 1, 1.25, 1.5, 2]

interface PlayerControlsProps {
  state: MediaPlayerState
  buffered: BufferedRange[]
  hasPrev: boolean
  hasNext: boolean
  tracks: SubtitleTrack[]
  selectedSubtitle: string | null
  subtitleDelay: number
  subtitleSize: SubtitleSize
  getThumbnail?: (time: number) => Promise<string | null>
  getCachedThumbnail?: (time: number) => string | null
  prefetchThumbnail?: (time: number) => void
  onToggle: () => void
  onSeek: (time: number) => void
  onPrev: () => void
  onNext: () => void
  onVolume: (volume: number) => void
  onToggleMute: () => void
  onRate: (rate: number) => void
  onToggleFullscreen: () => void
  onTogglePip: () => void
  onSelectSubtitle: (name: string | null) => void
  onSubtitleDelay: (delta: number) => void
  onResetSubtitleDelay: () => void
  onSubtitleSize: (size: SubtitleSize) => void
}

function iconButton(active = false) {
  return cn(
    'grid size-8 place-items-center rounded-[3px] text-white/80 transition-colors',
    'hover:bg-white/12 hover:text-white focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-white/60',
    'disabled:pointer-events-none disabled:opacity-30',
    active && 'bg-white/15 text-white',
  )
}

/** Prev/next live directly on the video: no button surface at all. */
function ghostIcon() {
  return cn(
    'grid size-8 place-items-center text-white/75 transition-[color,transform,opacity]',
    'hover:text-white active:scale-95 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-white/50',
    'disabled:pointer-events-none disabled:opacity-30',
  )
}

export function PlayerControls({
  state,
  buffered,
  hasPrev,
  hasNext,
  tracks,
  selectedSubtitle,
  subtitleDelay,
  subtitleSize,
  getThumbnail,
  getCachedThumbnail,
  prefetchThumbnail,
  onToggle,
  onSeek,
  onPrev,
  onNext,
  onVolume,
  onToggleMute,
  onRate,
  onToggleFullscreen,
  onTogglePip,
  onSelectSubtitle,
  onSubtitleDelay,
  onResetSubtitleDelay,
  onSubtitleSize,
}: PlayerControlsProps) {
  const [menu, setMenu] = useState<'captions' | 'settings' | null>(null)
  const menuRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!menu) return
    const onPointerDown = (event: PointerEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) setMenu(null)
    }
    window.addEventListener('pointerdown', onPointerDown)
    return () => window.removeEventListener('pointerdown', onPointerDown)
  }, [menu])

  const selectedTrack = tracks.find(track => track.name === selectedSubtitle) ?? null
  const currentSpeed = state.rate
  const volumeIcon = state.muted || state.volume === 0 ? VolumeX : state.volume < 0.5 ? Volume1 : Volume2
  const { elapsed, remaining } = formatPlaybackTimes(state.currentTime, state.duration)

  const cycleSubtitles = () => {
    if (tracks.length === 0) return
    if (!selectedSubtitle) {
      onSelectSubtitle(tracks[0].name)
      return
    }
    const index = tracks.findIndex(track => track.name === selectedSubtitle)
    onSelectSubtitle(index >= 0 && index + 1 < tracks.length ? tracks[index + 1].name : null)
  }

  return (
    <div className="absolute inset-x-0 bottom-0 z-20 bg-gradient-to-t from-black/90 via-black/50 to-transparent px-3 pb-3 pt-16 text-white sm:px-5 sm:pb-4">
      <Seekbar
        currentTime={state.currentTime}
        duration={state.duration}
        buffered={buffered}
        onSeek={onSeek}
        getThumbnail={getThumbnail}
        getCachedThumbnail={getCachedThumbnail}
        prefetchThumbnail={prefetchThumbnail}
      />

      <div className="mt-1 flex items-center gap-1 sm:gap-2">
        <button
          type="button"
          className={ghostIcon()}
          onClick={onToggle}
          aria-label={state.paused ? 'Play' : 'Pause'}
          title={state.paused ? 'Play (K)' : 'Pause (K)'}
        >
          {state.paused ? (
            <Play className="ml-0.5 size-5 fill-current" />
          ) : (
            <Pause className="size-5 fill-current" />
          )}
        </button>
        <button
          type="button"
          className={ghostIcon()}
          onClick={onPrev}
          disabled={!hasPrev}
          aria-label="Previous file"
          title="Previous file (B)"
        >
          <SkipBack className="size-5" />
        </button>
        <button
          type="button"
          className={ghostIcon()}
          onClick={onNext}
          disabled={!hasNext}
          aria-label="Next file"
          title="Next file (N)"
        >
          <SkipForward className="size-5" />
        </button>

        <div className="ml-1 flex items-center gap-1">
          <button
            type="button"
            className={iconButton()}
            onClick={onToggleMute}
            aria-label={state.muted ? 'Unmute' : 'Mute'}
            title="Mute (M)"
          >
            {(() => {
              const Icon = volumeIcon
              return <Icon className="size-5" />
            })()}
          </button>
          <input
            type="range"
            min={0}
            max={1}
            step={0.01}
            value={state.muted ? 0 : state.volume}
            onChange={event => onVolume(Number(event.target.value))}
            aria-label="Volume"
            className="hidden h-1 w-20 cursor-pointer appearance-none rounded-[2px] bg-white/25 accent-white sm:block [&::-webkit-slider-thumb]:size-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-[2px] [&::-webkit-slider-thumb]:bg-white"
          />
        </div>

        <span
          className="ml-1 whitespace-nowrap font-mono text-[11px] tabular text-white/85 sm:text-xs"
          title={`${formatTime(state.duration)} total`}
        >
          {elapsed}
          <span className="text-white/45"> / {remaining}</span>
        </span>

        {state.waiting && (
          <span className="ml-2 hidden items-center gap-1.5 text-[11px] text-white/60 sm:flex">
            <span className="size-1.5 animate-pulse rounded-full bg-[var(--accent)]" />
            buffering
          </span>
        )}

        <div className="ml-auto flex items-center gap-1" ref={menuRef}>
          <button
            type="button"
            className={iconButton(selectedTrack !== null)}
            onClick={cycleSubtitles}
            disabled={tracks.length === 0}
            aria-label="Cycle subtitles"
            title="Cycle subtitles (C)"
          >
            {selectedTrack ? <Captions className="size-5" /> : <CaptionsOff className="size-5" />}
          </button>
          <div className="relative">
            <button
              type="button"
              className={iconButton(menu === 'captions')}
              onClick={() => setMenu(menu === 'captions' ? null : 'captions')}
              disabled={tracks.length === 0}
              aria-label="Subtitle settings"
              title="Subtitle settings"
            >
              <Settings2 className="size-4.5" />
            </button>
            {menu === 'captions' && (
              <div className="absolute bottom-11 right-0 w-64 rounded-[3px] border border-white/15 bg-black/90 p-2 shadow-2xl backdrop-blur">
                <p className="px-2 pb-1.5 pt-1 text-[10px] font-semibold uppercase tracking-[0.12em] text-white/50">
                  Subtitles
                </p>
                <button
                  type="button"
                  onClick={() => onSelectSubtitle(null)}
                  className={cn(
                    'flex w-full items-center rounded-[1px] px-2 py-1.5 text-left text-xs hover:bg-white/10',
                    selectedSubtitle === null && 'text-[var(--accent)]',
                  )}
                >
                  Off
                </button>
                <div className="max-h-48 overflow-y-auto">
                  {tracks.map(track => (
                    <button
                      key={track.index}
                      type="button"
                      onClick={() => onSelectSubtitle(track.name)}
                      className={cn(
                        'flex w-full items-center rounded-[1px] px-2 py-1.5 text-left text-xs hover:bg-white/10',
                        selectedSubtitle === track.name && 'text-[var(--accent)]',
                      )}
                    >
                      {subtitleLabel(track)}
                    </button>
                  ))}
                </div>
                <div className="mt-1 flex items-center justify-between gap-1 border-t border-white/10 px-1 pt-2">
                  <span className="text-[10px] uppercase tracking-[0.1em] text-white/45">Delay</span>
                  <div className="flex items-center gap-1">
                    <button
                      type="button"
                      className="rounded-[1px] px-1.5 py-0.5 text-xs hover:bg-white/10"
                      onClick={() => onSubtitleDelay(-0.1)}
                      aria-label="Subtitle delay -0.1 seconds"
                    >
                      −.1
                    </button>
                    <button
                      type="button"
                      className="min-w-12 rounded-[1px] px-1 py-0.5 font-mono text-[11px] text-white/75 hover:bg-white/10"
                      onClick={onResetSubtitleDelay}
                      title="Reset subtitle delay"
                    >
                      {subtitleDelay > 0 ? '+' : ''}
                      {subtitleDelay.toFixed(1)}s
                    </button>
                    <button
                      type="button"
                      className="rounded-[1px] px-1.5 py-0.5 text-xs hover:bg-white/10"
                      onClick={() => onSubtitleDelay(0.1)}
                      aria-label="Subtitle delay +0.1 seconds"
                    >
                      +.1
                    </button>
                  </div>
                </div>
                <div className="mt-1 flex items-center justify-between gap-1 px-1 pb-1">
                  <span className="text-[10px] uppercase tracking-[0.1em] text-white/45">Size</span>
                  <div className="flex items-center gap-0.5">
                    {(['sm', 'md', 'lg'] as SubtitleSize[]).map(value => (
                      <button
                        key={value}
                        type="button"
                        onClick={() => onSubtitleSize(value)}
                        className={cn(
                          'rounded-[1px] px-1.5 py-0.5 text-[11px] uppercase hover:bg-white/10',
                          subtitleSize === value ? 'bg-[var(--accent)] text-[var(--accent-fg)]' : 'text-white/55',
                        )}
                      >
                        {value === 'sm' ? 'S' : value === 'md' ? 'M' : 'L'}
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            )}
          </div>

          <div className="relative">
            <button
              type="button"
              className={iconButton(menu === 'settings')}
              onClick={() => setMenu(menu === 'settings' ? null : 'settings')}
              aria-label="Playback settings"
              title="Playback settings"
            >
              <Gauge className="size-5" />
            </button>
            {menu === 'settings' && (
              <div className="absolute bottom-11 right-0 w-52 rounded-[3px] border border-white/15 bg-black/90 p-2 shadow-2xl backdrop-blur">
                <p className="px-2 pb-1.5 pt-1 text-[10px] font-semibold uppercase tracking-[0.12em] text-white/50">
                  Speed
                </p>
                <div className="flex flex-wrap gap-1 px-1">
                  {SPEEDS.map(speed => (
                    <button
                      key={speed}
                      type="button"
                      onClick={() => {
                        onRate(speed)
                        setMenu(null)
                      }}
                      className={cn(
                        'rounded-[1px] px-2 py-1 text-xs hover:bg-white/10',
                        Math.abs(currentSpeed - speed) < 0.01
                          ? 'bg-[var(--accent)] text-[var(--accent-fg)]'
                          : 'text-white/65',
                      )}
                    >
                      {speed}×
                    </button>
                  ))}
                </div>
                <div className="mt-2 border-t border-white/10 pt-2">
                  <button
                    type="button"
                    onClick={() => {
                      onTogglePip()
                      setMenu(null)
                    }}
                    className="flex w-full items-center gap-2 rounded-[1px] px-2 py-1.5 text-left text-xs text-white/80 hover:bg-white/10"
                  >
                    <PictureInPicture2 className="size-4" />
                    Picture in picture
                  </button>
                </div>
              </div>
            )}
          </div>

          <button
            type="button"
            className={iconButton()}
            onClick={onToggleFullscreen}
            aria-label={state.fullscreen ? 'Exit fullscreen' : 'Fullscreen'}
            title="Fullscreen (F)"
          >
            {state.fullscreen ? <Minimize className="size-5" /> : <Maximize className="size-5" />}
          </button>
        </div>
      </div>
    </div>
  )
}
