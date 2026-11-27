import { AlertTriangle, Copy, Eye, MonitorPlay, Music2, Play } from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button } from '@/components/ui/Button'
import { ProgressBar } from '@/components/ui/Progress'
import { FileTypeIcon, kindLabel, type FileKind } from '@/lib/fileIcons'
import { formatEta, formatPercent, formatSpeed } from '@/lib/format'
import type { useMediaPlayer } from '@/features/player/useMediaPlayer'
import type { DownloadSnapshot } from '@/types/api'

type Player = ReturnType<typeof useMediaPlayer>

interface PlayerSlotProps {
  download: DownloadSnapshot
  kind: FileKind
  streamUrl: string | null
  coverUrl: string | null
  audio: boolean
  image: boolean
  previewable: boolean
  dockActive: boolean
  preparing: boolean
  waitingForPeers: boolean
  error: boolean
  player: Player
  onRegisterSlot: (node: HTMLDivElement | null) => void
  onCopy: () => void
}

export function PlayerSlot({
  download,
  kind,
  streamUrl,
  coverUrl,
  audio,
  image,
  previewable,
  dockActive,
  preparing,
  waitingForPeers,
  error,
  player,
  onRegisterSlot,
  onCopy,
}: PlayerSlotProps) {
  const PreparingIcon = audio ? Music2 : image ? MonitorPlay : Play

  return (
    <div className="panel overflow-hidden">
      <div
        ref={onRegisterSlot}
        className="relative aspect-video w-full overflow-hidden bg-black"
        data-player-slot
      >
        {dockActive && coverUrl && (
          <img
            src={coverUrl}
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
              The stream is ready. Open it directly or copy the URL.
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
              <Button variant="secondary" size="sm" onClick={onCopy}>
                <Copy className="size-4" />
                Copy stream URL
              </Button>
            </div>
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
                {waitingForPeers
                  ? 'Waiting for peers. The preview starts automatically once data starts moving.'
                  : 'Waiting for the first pieces from the swarm. The preview starts automatically once the first bytes are available.'}
              </p>
            </div>
            <div className="w-56">
              <ProgressBar progress={download.progress} tone="stream" />
              <p className="mt-2 font-mono text-[10px] tabular text-[var(--muted)]">
                {formatPercent(download.progress)} · {formatSpeed(download.speed)} ·{' '}
                {download.peers} peers · {formatEta(download.etaSeconds)} left
              </p>
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
              The stream is ready. Open the file directly, or copy the stream URL.
            </p>
            <div className="mt-1 flex flex-wrap justify-center gap-2">
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
              <Button variant="secondary" size="sm" onClick={onCopy}>
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
          <Button variant="ghost" size="sm" onClick={onCopy}>
            <Copy className="size-3.5" />
            Copy URL
          </Button>
        </div>
      </div>
    </div>
  )
}
