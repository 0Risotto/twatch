import { Captions, ListVideo, SkipBack, SkipForward, Upload, X } from 'lucide-react'
import type { ChangeEvent, RefObject } from 'react'

import type { DownloadSnapshot, QueueFile } from '@/types/api'

import { cn } from '@/lib/cn'
import { type FileKind, FileTypeIcon, kindLabel } from '@/lib/fileIcons'
import { formatBytes, formatEta, formatSpeed, formatTime } from '@/lib/format'
import { subtitleLabel } from '@/lib/subtitles'

import { Button } from '@/components/ui/Button'
import { ProgressBar } from '@/components/ui/Progress'

import type { useMediaPlayer } from '@/features/player/useMediaPlayer'
import type { useSubtitleTracks } from '@/features/player/useSubtitleTracks'

type Player = ReturnType<typeof useMediaPlayer>
type Subtitles = ReturnType<typeof useSubtitleTracks>

interface PlayerSidebarProps {
  download: DownloadSnapshot
  kind: FileKind
  video: boolean
  audio: boolean
  image: boolean
  playable: boolean
  streamUrl: string | null
  bufferedAhead: number
  queue: QueueFile[]
  queueIndex: number
  hasPrev: boolean
  hasNext: boolean
  onPrev: () => void
  onNext: () => void
  onSwitch: (index: number) => void
  player: Player
  subtitles: Subtitles
  externalSubtitle: { name: string } | null
  setExternalSubtitle: (subtitle: null) => void
  subtitleInputRef: RefObject<HTMLInputElement | null>
  onLoadSubtitle: (event: ChangeEvent<HTMLInputElement>) => void
}

export function PlayerSidebar({
  download,
  kind,
  video,
  audio,
  image,
  playable,
  streamUrl,
  bufferedAhead,
  queue,
  queueIndex,
  hasPrev,
  hasNext,
  onPrev,
  onNext,
  onSwitch,
  player,
  subtitles,
  externalSubtitle,
  setExternalSubtitle,
  subtitleInputRef,
  onLoadSubtitle,
}: PlayerSidebarProps) {
  return (
    <aside className="space-y-3">
      {queue.length > 1 && (
        <section className="panel p-4">
          <div className="flex items-center justify-between gap-2">
            <h2 className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-[0.1em] text-[var(--muted-strong)]">
              <ListVideo className="size-3.5 text-[var(--accent)]" />
              Playback queue
            </h2>
            <span className="text-[10px] text-[var(--muted)]">
              {Math.max(1, queueIndex + 1)} / {queue.length}
            </span>
          </div>
          <div className="mt-2 flex items-center gap-1">
            <Button variant="ghost" size="sm" disabled={!hasPrev} onClick={onPrev}>
              <SkipBack className="size-3.5" />
              Prev
            </Button>
            <Button variant="ghost" size="sm" disabled={!hasNext} onClick={onNext}>
              Next
              <SkipForward className="size-3.5" />
            </Button>
          </div>
          <ul className="mt-2 max-h-56 space-y-0.5 overflow-y-auto">
            {queue.map((file, index) => (
              <li key={file.index}>
                <button
                  type="button"
                  onClick={() => onSwitch(file.index)}
                  disabled={index === queueIndex}
                  className={cn(
                    'flex w-full items-center gap-2 rounded-[2px] px-2 py-1.5 text-left text-[11px] transition-colors',
                    index === queueIndex
                      ? 'bg-[var(--accent-soft)] text-[var(--fg)]'
                      : 'text-[var(--muted)] hover:bg-[var(--surface-2)] hover:text-[var(--fg)]',
                  )}
                >
                  <span className="w-5 shrink-0 font-mono text-[10px] tabular opacity-60">
                    {index + 1}
                  </span>
                  <span className="min-w-0 flex-1 truncate">{file.name}</span>
                </button>
              </li>
            ))}
          </ul>
        </section>
      )}

      <section className="panel p-4">
        <h2 className="text-xs font-semibold uppercase tracking-[0.1em] text-[var(--muted-strong)]">
          Stream health
        </h2>
        <div className="mt-3 space-y-3">
          <Metric label="Progress" value={`${Math.round(download.progress * 100)}%`} />
          <ProgressBar
            progress={download.progress}
            tone={download.state === 'completed' ? 'success' : 'stream'}
            height={7}
          />
          <Metric label="Download" value={formatSpeed(download.speed)} />
          <Metric label="Peers" value={String(download.peers)} />
          <Metric
            label="Downloaded"
            value={`${formatBytes(download.downloaded)} / ${formatBytes(download.totalSize)}`}
          />
          <Metric label="ETA" value={formatEta(download.etaSeconds)} />
          {playable && player.state.duration > 0 && (
            <>
              <Metric
                label="Played"
                value={`${formatTime(player.state.currentTime)} / ${formatTime(
                  player.state.duration,
                )}`}
              />
              <Metric label="Buffered" value={`${formatTime(bufferedAhead)} ahead`} />
            </>
          )}
        </div>
      </section>

      {video && (
        <section className="panel p-4">
          <div className="flex items-center justify-between gap-2">
            <h2 className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-[0.1em] text-[var(--muted-strong)]">
              <Captions className="size-3.5 text-[var(--accent)]" />
              Subtitles
            </h2>
            <span className="text-[10px] text-[var(--muted)]">
              {subtitles.tracks.length} track{subtitles.tracks.length === 1 ? '' : 's'}
            </span>
          </div>
          <select
            value={subtitles.selected ?? 'off'}
            onChange={event => {
              const value = event.target.value
              subtitles.selectTrack(value === 'off' ? null : value)
            }}
            className="input mt-3 h-9 text-xs"
            aria-label="Subtitle track"
          >
            <option value="off">Off</option>
            {subtitles.tracks.map(track => (
              <option
                key={`${track.external ? 'external' : 'torrent'}:${track.index}`}
                value={track.name}
              >
                {subtitleLabel(track)}
                {track.external ? ' · external' : ''}
              </option>
            ))}
          </select>

          <div className="mt-2 flex flex-wrap items-center gap-2">
            <Button variant="secondary" size="sm" onClick={() => subtitleInputRef.current?.click()}>
              <Upload className="size-3.5" />
              Load subtitle file
            </Button>
            {externalSubtitle && (
              <Button variant="ghost" size="sm" onClick={() => setExternalSubtitle(null)}>
                <X className="size-3.5" />
                Remove external
              </Button>
            )}
            <input
              ref={subtitleInputRef}
              type="file"
              accept=".srt,.vtt,.ass,.ssa"
              className="hidden"
              onChange={onLoadSubtitle}
            />
          </div>

          <div className="mt-3 flex items-center justify-between gap-2">
            <span className="text-[10px] uppercase tracking-[0.1em] text-[var(--muted)]">
              Delay
            </span>
            <div className="flex items-center gap-1">
              <button
                type="button"
                className="btn btn-ghost h-7 px-2 text-[11px]"
                onClick={() => subtitles.adjustDelay(-1)}
              >
                −1s
              </button>
              <button
                type="button"
                className="btn btn-ghost h-7 px-2 text-[11px]"
                onClick={() => subtitles.adjustDelay(-0.1)}
              >
                −.1
              </button>
              <button
                type="button"
                className="min-w-12 font-mono text-[11px] text-[var(--fg)]"
                onClick={subtitles.resetDelay}
                title="Reset delay"
              >
                {subtitles.delay > 0 ? '+' : ''}
                {subtitles.delay.toFixed(1)}s
              </button>
              <button
                type="button"
                className="btn btn-ghost h-7 px-2 text-[11px]"
                onClick={() => subtitles.adjustDelay(0.1)}
              >
                +.1
              </button>
              <button
                type="button"
                className="btn btn-ghost h-7 px-2 text-[11px]"
                onClick={() => subtitles.adjustDelay(1)}
              >
                +1s
              </button>
            </div>
          </div>

          <div className="mt-2 flex items-center justify-between gap-2">
            <span className="text-[10px] uppercase tracking-[0.1em] text-[var(--muted)]">Size</span>
            <div className="flex items-center gap-1">
              {subtitles.selectedIsAss ? (
                <span className="text-[10px] text-[var(--muted)]">from release</span>
              ) : (
                (['sm', 'md', 'lg'] as const).map(value => (
                  <button
                    key={value}
                    type="button"
                    onClick={() => subtitles.setSize(value)}
                    className={cn(
                      'rounded-[2px] px-2 py-0.5 text-[11px] uppercase',
                      subtitles.size === value
                        ? 'bg-[var(--accent-soft)] text-[var(--fg)]'
                        : 'text-[var(--muted)] hover:bg-[var(--surface-2)]',
                    )}
                  >
                    {value === 'sm' ? 'S' : value === 'md' ? 'M' : 'L'}
                  </button>
                ))
              )}
            </div>
          </div>
          <p className="mt-2 text-[10px] leading-relaxed text-[var(--muted)]">
            Tracks come from this torrent or an external file. SRT/VTT use native tracks; ASS/SSA
            renders through libass. Accepted external formats: SRT, VTT, ASS, SSA. Your track, delay
            and size are remembered per torrent.
          </p>
        </section>
      )}

      <section className="panel p-4">
        <h2 className="text-xs font-semibold uppercase tracking-[0.1em] text-[var(--muted-strong)]">
          File
        </h2>
        <p className="mt-2 flex items-start gap-2 break-all font-mono text-[11px] leading-relaxed text-[var(--muted)]">
          <FileTypeIcon
            name={download.fileNames[0] ?? download.label}
            className="mt-0.5 size-4 shrink-0"
          />
          <span>{download.fileNames.join(', ')}</span>
        </p>
        {streamUrl && (
          <p className="mt-3 break-all font-mono text-[10px] text-[var(--faint)]">{streamUrl}</p>
        )}
      </section>

      <section className="panel p-4">
        <h2 className="text-xs font-semibold uppercase tracking-[0.1em] text-[var(--muted-strong)]">
          {kindLabel(kind)} preview
        </h2>
        <p className="mt-2 text-[11px] leading-relaxed text-[var(--muted)]">
          {video
            ? 'MP4 and WebM play inline. MKV and HEVC are prepared by the local transcoder and use the same player controls.'
            : audio
              ? 'Audio is played with the browser engine. MP3, AAC, FLAC and Opus usually work.'
              : image
                ? 'Images are served through the same byte-range stream. Click the image to open the original resolution in a new tab.'
                : 'This file type is not rendered in the browser. Open it directly.'}
        </p>
      </section>
    </aside>
  )
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-3 text-[11px]">
      <span className="text-[var(--muted)]">{label}</span>
      <span className="font-mono tabular text-[var(--fg)]">{value}</span>
    </div>
  )
}
