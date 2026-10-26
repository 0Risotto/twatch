import { useEffect, useMemo, useRef, useState } from 'react'
import { Link, useLocation, useNavigate, useSearchParams } from 'react-router-dom'
import {
  AlertTriangle,
  ArrowLeft,
  Captions,
  Copy,
  ExternalLink,
  Eye,
  FileImage,
  MonitorPlay,
  Music2,
  Play,
  Tv,
} from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button } from '@/components/ui/Button'
import { EmptyState } from '@/components/ui/EmptyState'
import { ProgressBar } from '@/components/ui/Progress'
import { PageHeader } from '@/components/PageHeader'
import { api } from '@/lib/api'
import { formatBytes, formatEta, formatSpeed } from '@/lib/format'
import { canPreview, fileKind, isAudio, isImage, isVideo, kindLabel, mediaKind, FileTypeIcon } from '@/lib/fileIcons'
import { useCover } from '@/lib/covers'
import { matchingSubtitles, subtitleLabel, subtitleLanguage, subtitleUrl } from '@/lib/subtitles'
import { useAppStore } from '@/store/app'
import { cn } from '@/lib/cn'

export function PlayerPage() {
  const [params] = useSearchParams()
  const navigate = useNavigate()
  const location = useLocation()
  const downloads = useAppStore(state => state.downloads)
  const player = useAppStore(state => state.player)
  const addToast = useAppStore(state => state.addToast)
  const [mediaError, setMediaError] = useState(false)
  const [busy, setBusy] = useState<string | null>(null)
  const [subtitle, setSubtitle] = useState<number | null>(null)
  const videoRef = useRef<HTMLVideoElement>(null)

  const requestedId = Number(params.get('download')) || null

  const download = useMemo(() => {
    if (requestedId) {
      const exact = downloads.find(item => item.id === requestedId)
      if (exact) return exact
    }
    if (player.downloadId) {
      const playing = downloads.find(item => item.id === player.downloadId)
      if (playing) return playing
    }
    return (
      downloads.find(item => item.streamPath) ??
      downloads.find(item => item.isStreaming) ??
      null
    )
  }, [downloads, player.downloadId, requestedId])

  const cover = useCover(
    download?.torrentName,
    download ? mediaKind(download.torrentName) : undefined,
  )
  const tracks = download?.subtitles ?? []
  const matching = useMemo(
    () => matchingSubtitles(download?.fileNames[0] ?? download?.label ?? '', tracks),
    [download?.fileNames, download?.label, tracks],
  )
  const defaultSubtitle =
    (matching.find(track => subtitleLanguage(track.name).code.startsWith('en')) ?? matching[0])?.index ??
    null

  useEffect(() => {
    setMediaError(false)
  }, [download?.streamPath])

  useEffect(() => {
    setSubtitle(defaultSubtitle)
  }, [download?.id, defaultSubtitle])

  const syncTextTracks = () => {
    const video = videoRef.current
    if (!video) return
    for (let index = 0; index < video.textTracks.length; index += 1) {
      const track = tracks[index]
      video.textTracks[index].mode =
        track && track.index === subtitle ? 'showing' : 'disabled'
    }
  }

  useEffect(() => {
    syncTextTracks()
  }, [subtitle, tracks, download?.streamPath])

  const call = async (name: string, action: () => Promise<unknown>) => {
    if (busy) return
    setBusy(name)
    try {
      await action()
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Playback action failed')
    } finally {
      setBusy(null)
    }
  }

  const openExternal = () => {
    if (!download) return
    void call('external', async () => {
      await api.openPlayer(download.id)
      addToast('info', 'Opened in the external player')
    })
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
    if (!download?.streamPath) return
    try {
      await navigator.clipboard.writeText(`${window.location.origin}${download.streamPath}`)
      addToast('success', 'Stream URL copied')
    } catch {
      addToast('warning', 'Clipboard is unavailable')
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

  const preparing = !download.streamPath && download.state !== 'failed'
  const streamUrl = download.streamPath
  const external = player.running && player.downloadId === download.id
  const kind = fileKind(download.fileNames[0] ?? download.label)
  const video = isVideo(kind)
  const audio = isAudio(kind)
  const image = isImage(kind)
  const previewable = canPreview(kind)
  const externalPlayable = video || audio
  const PreparingIcon = audio ? Music2 : image ? FileImage : Play

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
              <Button variant="secondary" size="sm" loading={busy === 'external'} onClick={openExternal}>
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

      <div className="grid gap-4 xl:grid-cols-[1fr_300px]">
        <div className="panel overflow-hidden">
          <div className="relative aspect-video bg-black">
            {streamUrl && !mediaError ? (
              video ? (
                <video
                  key={streamUrl}
                  ref={videoRef}
                  src={streamUrl}
                  poster={cover?.url ?? undefined}
                  controls
                  autoPlay
                  playsInline
                  className="h-full w-full bg-black"
                  onLoadedMetadata={syncTextTracks}
                  onError={() => setMediaError(true)}
                >
                  {download.torrentId !== null &&
                    tracks.map(track => (
                      <track
                        key={track.index}
                        kind="subtitles"
                        src={subtitleUrl(download.torrentId!, track.index)}
                        srcLang={subtitleLanguage(track.name).code}
                        label={subtitleLabel(track)}
                        default={track.index === subtitle}
                      />
                    ))}
                </video>
              ) : audio ? (
                <div className="flex h-full flex-col items-center justify-center gap-5 bg-[radial-gradient(ellipse_at_center,color-mix(in_srgb,var(--warning)_12%,transparent),transparent_65%)] p-6">
                  {cover?.url ? (
                    <img
                      src={cover.url}
                      alt=""
                      referrerPolicy="no-referrer"
                      className="h-48 w-32 rounded-[var(--radius-lg)] border border-[var(--border)] object-cover shadow-[var(--shadow-lg)]"
                    />
                  ) : (
                    <div className="relative grid size-32 place-items-center rounded-full border border-[var(--border-strong)] bg-[var(--surface-2)] shadow-[var(--shadow-md)]">
                      <div className="absolute inset-2 rounded-full border border-[var(--border)]" />
                      <Music2 className="size-12 text-[var(--warning)]" />
                    </div>
                  )}
                  <div className="max-w-md text-center">
                    <p className="truncate text-sm font-semibold text-[var(--fg)]">{download.label}</p>
                    <p className="mt-1 truncate text-xs text-[var(--muted)]">{download.torrentName}</p>
                  </div>
                  <audio
                    key={streamUrl}
                    src={streamUrl}
                    controls
                    autoPlay
                    className="w-full max-w-md"
                    onError={() => setMediaError(true)}
                  />
                </div>
              ) : image ? (
                <a
                  href={streamUrl}
                  target="_blank"
                  rel="noreferrer"
                  className="flex h-full items-center justify-center bg-[var(--bg)] p-2"
                  title="Open original image"
                >
                  <img
                    src={streamUrl}
                    alt={download.label}
                    className="max-h-full max-w-full object-contain"
                    onError={() => setMediaError(true)}
                  />
                </a>
              ) : (
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
              )
            ) : preparing ? (
              <div className="flex h-full flex-col items-center justify-center gap-4 text-center">
                <div className="relative grid size-20 place-items-center rounded-full border border-[var(--border-strong)] bg-[var(--surface-2)]">
                  <PreparingIcon className="size-8 text-[var(--accent)]" />
                  <span className="absolute inset-0 animate-ping rounded-full border border-[var(--accent)] opacity-30" />
                </div>
                <div>
                  <p className="text-sm font-semibold text-[var(--fg)]">Preparing {previewable ? kindLabel(kind).toLowerCase() : 'file'}…</p>
                  <p className="mt-1 max-w-sm text-xs leading-relaxed text-[var(--muted)]">
                    Waiting for the first pieces from the swarm. The preview starts automatically once the
                    first bytes are available.
                  </p>
                </div>
                <div className="w-56">
                  <ProgressBar progress={download.progress} tone="stream" />
                </div>
              </div>
            ) : (
              <div className="flex h-full flex-col items-center justify-center gap-3 px-6 text-center">
                <AlertTriangle className="size-7 text-[var(--warning)]" />
                <p className="text-sm font-semibold text-[var(--fg)]">
                  {image ? 'Could not load this image' : audio ? 'This audio codec cannot play in the browser' : 'This video container cannot play in the browser'}
                </p>
                <p className="max-w-md text-xs leading-relaxed text-[var(--muted)]">
                  The stream is ready. {externalPlayable
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
            <div className="ml-auto flex items-center gap-2">
              {external && <span className="text-[11px] text-[var(--badge-stream)]">Playing in external player</span>}
              <Button variant="ghost" size="sm" onClick={() => void copyStream()}>
                <Copy className="size-3.5" />
                Copy URL
              </Button>
            </div>
          </div>
        </div>

        <aside className="space-y-3">
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
            </div>
          </section>

          {video && tracks.length > 0 && (
            <section className="panel p-4">
              <div className="flex items-center justify-between gap-2">
                <h2 className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-[0.1em] text-[var(--muted-strong)]">
                  <Captions className="size-3.5 text-[var(--accent)]" />
                  Subtitles
                </h2>
                <span className="text-[10px] text-[var(--muted)]">{tracks.length} track{tracks.length === 1 ? '' : 's'}</span>
              </div>
              <select
                value={subtitle ?? 'off'}
                onChange={event => {
                  const value = event.target.value
                  setSubtitle(value === 'off' ? null : Number(value))
                }}
                className="input mt-3 h-9 text-xs"
                aria-label="Subtitle track"
              >
                <option value="off">Off</option>
                {tracks.map(track => (
                  <option key={track.index} value={track.index}>
                    {subtitleLabel(track)}
                    {matching.some(item => item.index === track.index) ? ' · matched' : ''}
                  </option>
                ))}
              </select>
              <p className="mt-2 text-[10px] leading-relaxed text-[var(--muted)]">
                Tracks are read from this torrent. SRT, ASS/SSA and VTT are converted to WebVTT on
                the fly for the browser player.
              </p>
            </section>
          )}

          <section className="panel p-4">
            <h2 className="text-xs font-semibold uppercase tracking-[0.1em] text-[var(--muted-strong)]">
              File
            </h2>
            <p className="mt-2 flex items-start gap-2 break-all font-mono text-[11px] leading-relaxed text-[var(--muted)]">
              <FileTypeIcon name={download.fileNames[0] ?? download.label} className="mt-0.5 size-4 shrink-0" />
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
                ? 'MP4/H.264 and WebM play inline. For MKV, HEVC, DTS, or subtitle-heavy releases, use mpv/VLC — the same byte stream is handed to the player.'
                : audio
                  ? 'Audio is played with the browser’s native audio element. MP3, AAC, FLAC and Opus usually work; DTS and exotic codecs may need a native player.'
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
