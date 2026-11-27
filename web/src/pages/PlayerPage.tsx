import { useEffect, useRef, useState } from 'react'
import type { ChangeEvent as ReactChangeEvent } from 'react'
import { Link, useLocation, useNavigate, useSearchParams } from 'react-router-dom'
import {
  AlertTriangle,
  ArrowLeft,
  Captions,
  Copy,
  ExternalLink,
  Eye,
  ListVideo,
  MonitorPlay,
  Music2,
  Play,
  SkipBack,
  SkipForward,
  Tv,
  Upload,
  X,
} from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button } from '@/components/ui/Button'
import { EmptyState } from '@/components/ui/EmptyState'
import { ProgressBar } from '@/components/ui/Progress'
import { PageHeader } from '@/components/PageHeader'
import { api } from '@/lib/api'
import { formatBytes, formatEta, formatSpeed, formatTime } from '@/lib/format'
import {
  FileTypeIcon,
  canPreview,
  fileKind,
  isAudio,
  isImage,
  isInlineVideoName,
  isVideo,
  kindLabel,
} from '@/lib/fileIcons'
import { subtitleLabel } from '@/lib/subtitles'
import { useAppStore } from '@/store/app'
import { cn } from '@/lib/cn'
import { usePlayback } from '@/features/player/PlaybackProvider'
import type { SubtitleSize } from '@/features/player/useSubtitleTracks'

export function PlayerPage() {
  const [params] = useSearchParams()
  const navigate = useNavigate()
  const location = useLocation()
  const downloads = useAppStore(state => state.downloads)
  const playerState = useAppStore(state => state.player)
  const addToast = useAppStore(state => state.addToast)
  const [busy, setBusy] = useState<string | null>(null)
  const subtitleInputRef = useRef<HTMLInputElement | null>(null)
  const {
    download: activeDownload,
    cover,
    player,
    subtitles,
    externalSubtitle,
    setExternalSubtitle,
    switchTo,
    playPrev,
    playNext,
    hasPrev,
    hasNext,
    activate,
    registerSlot,
  } = usePlayback()

  const requestedId = Number(params.get('download')) || null
  const autoDownload =
    !requestedId && !activeDownload
      ? (downloads.find(item => item.streamPath) ??
        downloads.find(item => item.isStreaming) ??
        null)
      : null
  const autoId = autoDownload?.id ?? null

  useEffect(() => {
    if (requestedId) activate(requestedId)
  }, [activate, requestedId])

  useEffect(() => {
    if (autoId) activate(autoId)
  }, [activate, autoId])

  const download = requestedId
    ? (downloads.find(item => item.id === requestedId) ?? activeDownload)
    : (activeDownload ?? autoDownload)

  const streamUrl = download?.streamPath ?? null
  const kind = fileKind(download?.fileNames[0] ?? download?.label ?? '')
  const video = isVideo(kind)
  const audio = isAudio(kind)
  const image = isImage(kind)
  const previewable = canPreview(kind)
  const externalPlayable = video || audio
  const playable = video || audio
  const inlineVideo = video && isInlineVideoName(download?.fileNames[0] ?? download?.label ?? '')
  const externalOnly = video && !inlineVideo
  const queue = download?.queue ?? []
  const queueIndex = queue.findIndex(item => item.index === (download?.fileIndex ?? 0))
  const preparing = Boolean(download) && !streamUrl && download?.state !== 'failed'
  const error = Boolean(download) && (player.state.error || download?.state === 'failed')
  const dockActive = Boolean(download && streamUrl && (audio || inlineVideo) && !error)
  const external = Boolean(
    download && playerState.running && playerState.downloadId === download.id,
  )

  const openExternal = () => {
    if (!download) return
    if (busy) return
    setBusy('external')
    api
      .openPlayer(download.id)
      .then(() => addToast('info', 'Opened in the external player'))
      .catch((reason: unknown) =>
        addToast('error', reason instanceof Error ? reason.message : 'Playback action failed'),
      )
      .finally(() => setBusy(null))
  }

  const goBack = () => {
    const from = (location.state as { from?: string } | null)?.from
    if (from) {
      navigate(from)
      return
    }
    if (window.history.length > 1) navigate(-1)
    else navigate('/downloads')
  }

  const copyStream = async () => {
    if (!streamUrl) return
    try {
      await navigator.clipboard.writeText(`${window.location.origin}${streamUrl}`)
      addToast('success', 'Stream URL copied')
    } catch {
      addToast('warning', 'Clipboard is unavailable')
    }
  }

  const loadSubtitle = async (event: ReactChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    event.target.value = ''
    if (!file) return
    if (!/\.(srt|vtt|ass|ssa)$/i.test(file.name)) {
      addToast('warning', 'Choose an SRT, VTT, ASS or SSA subtitle file')
      return
    }
    try {
      const content = await file.text()
      setExternalSubtitle({ name: file.name, content })
      addToast('success', `Loaded ${file.name}`)
    } catch {
      addToast('error', 'Could not read that subtitle file')
    }
  }

  if (!download) {
    return (
      <div className="space-y-5">
        <PageHeader
          eyebrow="Playback"
          title="Now playing"
          description="Inline playback appears here while the engine streams pieces from the swarm."
          actions={
            <Link to="/downloads" className="btn btn-secondary h-8 px-3 text-xs">
              Downloads
            </Link>
          }
        />
        <EmptyState
          icon={<MonitorPlay className="size-6" />}
          title="Nothing playing"
          description="Pick a file in a torrent and press Watch. twatch starts a one-file stream and keeps downloading in the background."
          action={
            <Link to="/search" className="btn btn-secondary">
              Find a torrent
              <Play className="size-4" />
            </Link>
          }
        />
      </div>
    )
  }

  const PreparingIcon = audio ? Music2 : image ? MonitorPlay : Play

  return (
    <div className="space-y-4">
      <PageHeader
        eyebrow="Playback"
        title={download.label}
        description={`${download.torrentName} · ${formatBytes(download.totalSize)}`}
        actions={
          <>
            <Button variant="ghost" size="sm" onClick={goBack}>
              <ArrowLeft className="size-4" />
              Back
            </Button>
            <Link to="/downloads" className="btn btn-ghost h-8 px-3 text-xs">
              Downloads
            </Link>
            {externalPlayable && (
              <Button
                variant="secondary"
                size="sm"
                loading={busy === 'external'}
                onClick={openExternal}
              >
                <Tv className="size-4" />
                Open in mpv / VLC
              </Button>
            )}
            {streamUrl && !previewable && (
              <a
                href={streamUrl}
                target="_blank"
                rel="noreferrer"
                className="btn btn-secondary h-8 px-3 text-xs"
              >
                <Eye className="size-4" />
                Open file
              </a>
            )}
          </>
        }
      />

      <div className="grid gap-4 xl:grid-cols-[1fr_320px]">
        <div className="panel overflow-hidden">
          <div
            ref={registerSlot}
            className="relative aspect-video w-full overflow-hidden bg-black"
            data-player-slot
          >
            {dockActive && cover?.url && (
              <img
                src={cover.url}
                alt=""
                referrerPolicy="no-referrer"
                className="h-full w-full object-cover opacity-20"
              />
            )}

            {streamUrl && !error && image && (
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

            {streamUrl && !error && !previewable && !image && (
              <div className="flex h-full flex-col items-center justify-center gap-3 px-6 text-center">
                <FileTypeIcon name={download.fileNames[0] ?? download.label} className="size-10" />
                <p className="text-sm font-semibold text-[var(--fg)]">
                  {kindLabel(kind)} preview is not available in the browser
                </p>
                <p className="max-w-md text-xs leading-relaxed text-[var(--muted)]">
                  The stream is ready. Open it directly, copy the URL, or hand it to a native app.
                </p>
                <div className="mt-1 flex flex-wrap justify-center gap-2">
                  <a
                    href={streamUrl}
                    target="_blank"
                    rel="noreferrer"
                    className="btn btn-primary h-8 px-3 text-xs"
                  >
                    <Eye className="size-4" />
                    Open file
                  </a>
                  <Button variant="secondary" size="sm" onClick={() => void copyStream()}>
                    <Copy className="size-4" />
                    Copy stream URL
                  </Button>
                </div>
              </div>
            )}

            {externalOnly && streamUrl && !error && (
              <div className="flex h-full flex-col items-center justify-center gap-3 px-6 text-center">
                <Tv className="size-8 text-[var(--accent)]" />
                <p className="text-sm font-semibold text-[var(--fg)]">
                  This video needs a native player
                </p>
                <p className="max-w-md text-xs leading-relaxed text-[var(--muted)]">
                  {download.fileNames[0] ?? download.label} uses a container or codec Chromium
                  cannot decode reliably. The stream is ready.
                </p>
                <Button variant="primary" size="sm" onClick={openExternal}>
                  <ExternalLink className="size-4" />
                  Open in mpv / VLC
                </Button>
              </div>
            )}

            {preparing && (
              <div className="flex h-full flex-col items-center justify-center gap-4 text-center">
                <div className="relative grid size-20 place-items-center rounded-full border border-[var(--border-strong)] bg-[var(--surface-2)]">
                  <PreparingIcon className="size-8 text-[var(--accent)]" />
                  <span className="absolute inset-0 animate-ping rounded-full border border-[var(--accent)] opacity-30" />
                </div>
                <div>
                  <p className="text-sm font-semibold text-[var(--fg)]">
                    Preparing {previewable ? kindLabel(kind).toLowerCase() : 'file'}…
                  </p>
                  <p className="mt-1 max-w-sm text-xs leading-relaxed text-[var(--muted)]">
                    Waiting for the first pieces from the swarm. The preview starts automatically
                    once the first bytes are available.
                  </p>
                </div>
                <div className="w-56">
                  <ProgressBar progress={download.progress} tone="stream" />
                </div>
              </div>
            )}

            {error && !dockActive && (
              <div className="flex h-full flex-col items-center justify-center gap-3 px-6 text-center">
                <AlertTriangle className="size-7 text-[var(--warning)]" />
                <p className="text-sm font-semibold text-[var(--fg)]">
                  {image
                    ? 'Could not load this image'
                    : audio
                      ? 'This audio codec cannot play in the browser'
                      : 'This video container cannot play in the browser'}
                </p>
                <p className="max-w-md text-xs leading-relaxed text-[var(--muted)]">
                  The stream is ready.{' '}
                  {externalPlayable
                    ? 'Hand it to mpv or VLC, or download the file and play it locally.'
                    : 'Open the file directly, or copy the stream URL.'}
                </p>
                <div className="mt-1 flex flex-wrap justify-center gap-2">
                  {externalPlayable && (
                    <Button variant="primary" size="sm" onClick={openExternal}>
                      <Tv className="size-4" />
                      Open in mpv / VLC
                    </Button>
                  )}
                  {streamUrl && (
                    <Button variant="secondary" size="sm" onClick={player.retry}>
                      <Play className="size-4" />
                      Retry
                    </Button>
                  )}
                  {streamUrl && (
                    <a
                      href={streamUrl}
                      target="_blank"
                      rel="noreferrer"
                      className="btn btn-secondary h-8 px-3 text-xs"
                    >
                      <Eye className="size-4" />
                      Open file
                    </a>
                  )}
                  <Button variant="secondary" size="sm" onClick={() => void copyStream()}>
                    <Copy className="size-4" />
                    Copy stream URL
                  </Button>
                </div>
              </div>
            )}
          </div>

          <div className="flex flex-wrap items-center gap-3 border-t border-[var(--border-soft)] px-4 py-3">
            <Badge tone={download.state === 'completed' ? 'success' : 'stream'} dot>
              {download.state === 'completed' ? 'ready' : 'streaming'}
            </Badge>
            <span className="font-mono text-[11px] tabular text-[var(--muted)]">
              {Math.round(download.progress * 100)}% · {formatSpeed(download.speed)}
            </span>
            {player.state.resumed && (
              <span className="rounded-full border border-[var(--border-soft)] bg-[var(--surface-2)] px-2 py-0.5 text-[10px] text-[var(--muted)]">
                resumed
              </span>
            )}
            <div className="ml-auto flex items-center gap-2">
              {external && (
                <span className="text-[11px] text-[var(--badge-stream)]">
                  Playing in external player
                </span>
              )}
              <Button variant="ghost" size="sm" onClick={() => void copyStream()}>
                <Copy className="size-3.5" />
                Copy URL
              </Button>
            </div>
          </div>
        </div>

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
                <Button variant="ghost" size="sm" disabled={!hasPrev} onClick={playPrev}>
                  <SkipBack className="size-3.5" />
                  Prev
                </Button>
                <Button variant="ghost" size="sm" disabled={!hasNext} onClick={playNext}>
                  Next
                  <SkipForward className="size-3.5" />
                </Button>
              </div>
              <ul className="mt-2 max-h-56 space-y-0.5 overflow-y-auto">
                {queue.map((file, index) => (
                  <li key={file.index}>
                    <button
                      type="button"
                      onClick={() => void switchTo(file.index)}
                      disabled={busy !== null || index === queueIndex}
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
                <Metric
                  label="Played"
                  value={`${formatTime(player.state.currentTime)} / ${formatTime(
                    player.state.duration,
                  )}`}
                />
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
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => subtitleInputRef.current?.click()}
                >
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
                  onChange={event => void loadSubtitle(event)}
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
                <span className="text-[10px] uppercase tracking-[0.1em] text-[var(--muted)]">
                  Size
                </span>
                <div className="flex items-center gap-1">
                  {subtitles.selectedIsAss ? (
                    <span className="text-[10px] text-[var(--muted)]">from release</span>
                  ) : (
                    (['sm', 'md', 'lg'] as SubtitleSize[]).map(value => (
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
                Tracks come from this torrent or an external file. SRT/VTT use native tracks;
                ASS/SSA renders through libass. Accepted external formats: SRT, VTT, ASS, SSA. Your
                track, delay and size are remembered per torrent.
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
              <p className="mt-3 break-all font-mono text-[10px] text-[var(--faint)]">
                {streamUrl}
              </p>
            )}
          </section>

          <section className="panel p-4">
            <h2 className="text-xs font-semibold uppercase tracking-[0.1em] text-[var(--muted-strong)]">
              {kindLabel(kind)} preview
            </h2>
            <p className="mt-2 text-[11px] leading-relaxed text-[var(--muted)]">
              {video
                ? 'MP4/H.264 and WebM play inline. For MKV, HEVC, DTS, or subtitle-heavy releases, use mpv/VLC — the same byte stream is handed to the player.'
                : audio
                  ? 'Audio is played with the browser engine. MP3, AAC, FLAC and Opus usually work; DTS and exotic codecs may need a native player.'
                  : image
                    ? 'Images are served through the same byte-range stream. Click the image to open the original resolution in a new tab.'
                    : 'This file type is not rendered in the browser. Open it directly or hand it to a native application.'}
            </p>
            {externalPlayable && (
              <Button variant="secondary" size="sm" className="mt-3" onClick={openExternal}>
                <ExternalLink className="size-3.5" />
                Hand off to player
              </Button>
            )}
          </section>
        </aside>
      </div>
    </div>
  )
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-3 text-[11px]">
      <span className="text-[var(--muted)]">{label}</span>
      <span className={cn('font-mono tabular text-[var(--fg)]')}>{value}</span>
    </div>
  )
}
