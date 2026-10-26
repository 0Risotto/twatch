import { useEffect, useRef, useState } from 'react'
import { motion } from 'motion/react'
import { Link, useNavigate } from 'react-router-dom'
import {
  ArrowDownToLine,
  ArrowUpRight,
  Captions,
  Copy,
  Eye,
  Pause,
  Play,
  RotateCcw,
  Trash2,
  Tv,
  XCircle,
} from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button, IconButton } from '@/components/ui/Button'
import { EmptyState } from '@/components/ui/EmptyState'
import { ProgressRing, Sparkline } from '@/components/ui/Progress'
import { SegmentedControl } from '@/components/ui/SegmentedControl'
import { PageHeader, SectionHeader } from '@/components/PageHeader'
import { api } from '@/lib/api'
import { formatBytes, formatEta, formatPercent, formatSpeed } from '@/lib/format'
import { canPreview, fileKind, previewLabel, FileTypeIcon } from '@/lib/fileIcons'
import { CoverArt } from '@/components/CoverArt'
import { springs } from '@/lib/motion'
import { useAppStore } from '@/store/app'
import type { DownloadSnapshot } from '@/types/api'
import { cn } from '@/lib/cn'

type Tab = 'active' | 'completed' | 'failed'

function useSpeedSamples(speed: number, active: boolean): number[] {
  const [samples, setSamples] = useState<number[]>([])
  const speedRef = useRef(speed)

  useEffect(() => {
    speedRef.current = speed
  }, [speed])

  useEffect(() => {
    if (!active) return
    const timer = window.setInterval(() => {
      setSamples(current => [...current.slice(-39), speedRef.current])
    }, 1000)
    return () => window.clearInterval(timer)
  }, [active])

  return samples
}

const toneForState = (state: DownloadSnapshot['state']) => {
  switch (state) {
    case 'completed':
      return 'success' as const
    case 'failed':
      return 'danger' as const
    case 'paused':
      return 'warning' as const
    case 'streaming':
      return 'stream' as const
    default:
      return 'accent' as const
  }
}

function DownloadCard({ download }: { download: DownloadSnapshot }) {
  const addToast = useAppStore(state => state.addToast)
  const removeDownload = useAppStore(state => state.optimisticRemoveDownload)
  const navigate = useNavigate()
  const [busy, setBusy] = useState<string | null>(null)
  const active = download.state === 'downloading' || download.state === 'streaming'
  const samples = useSpeedSamples(download.speed, active)
  const kind = fileKind(download.fileNames[0] ?? download.label)
  const previewable = canPreview(kind)
  const externalPlayable = kind === 'movie' || kind === 'tv' || kind === 'audio'
  const PreviewIcon = kind === 'image' ? Eye : Play

  const call = async (name: string, action: () => Promise<unknown>) => {
    if (busy) return
    setBusy(name)
    try {
      await action()
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Action failed')
    } finally {
      setBusy(null)
    }
  }

  const pause = () =>
    void call(download.state === 'paused' ? 'resume' : 'pause', () =>
      download.state === 'paused' ? api.resumeDownload(download.id) : api.pauseDownload(download.id),
    )

  const remove = () =>
    void call('remove', async () => {
      await api.removeDownload(download.id)
      removeDownload(download.id)
      addToast('success', 'Removed from activity')
    })

  const openExternal = () =>
    void call('open', async () => {
      await api.openPlayer(download.id)
      addToast('info', 'Opened in the external player')
    })

  const copyStream = async () => {
    if (!download.streamPath) return
    try {
      await navigator.clipboard.writeText(`${window.location.origin}${download.streamPath}`)
      addToast('success', 'Stream URL copied')
    } catch {
      addToast('warning', 'Clipboard is unavailable')
    }
  }

  return (
    <motion.article
      layout
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.98 }}
      transition={springs.smooth}
      className="panel overflow-hidden"
    >
      <div className="flex items-start gap-4 p-4">
        <CoverArt title={download.torrentName} className="hidden h-[68px] w-[46px] shrink-0 sm:block" />
        <ProgressRing progress={download.progress} size={64} stroke={5}>
          <span className="font-mono text-[13px] font-semibold tabular text-[var(--fg)]">
            {formatPercent(download.progress)}
          </span>
        </ProgressRing>

        <div className="min-w-0 flex-1">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0">
              <h3 className="flex min-w-0 items-center gap-2 truncate text-sm font-semibold text-[var(--fg)]">
                <FileTypeIcon name={download.fileNames[0] ?? download.label} className="size-4" />
                <span className="truncate">{download.label}</span>
              </h3>
              <p className="mt-0.5 truncate text-[11px] text-[var(--muted)]">{download.torrentName}</p>
            </div>
            <div className="flex shrink-0 items-center gap-1.5">
              {(download.subtitles?.length ?? 0) > 0 && (
                <Badge tone="neutral">
                  <Captions className="size-3" />
                  {download.subtitles?.length ?? 0} CC
                </Badge>
              )}
              <Badge
                tone={
                  download.state === 'completed'
                    ? 'success'
                    : download.state === 'failed'
                      ? 'danger'
                      : download.state === 'paused'
                        ? 'warning'
                        : download.isStreaming
                          ? 'stream'
                          : 'download'
                }
                dot
              >
                {download.state === 'streaming' ? 'streaming' : download.state}
              </Badge>
            </div>
          </div>

          <div className="mt-3 grid grid-cols-2 gap-x-4 gap-y-2 text-[11px] sm:grid-cols-4">
            <div>
              <p className="text-[9px] uppercase tracking-wider text-[var(--faint)]">Speed</p>
              <p className="mt-0.5 font-mono tabular text-[var(--fg)]">{formatSpeed(download.speed)}</p>
            </div>
            <div>
              <p className="text-[9px] uppercase tracking-wider text-[var(--faint)]">Peers</p>
              <p className="mt-0.5 font-mono tabular text-[var(--fg)]">{download.peers}</p>
            </div>
            <div>
              <p className="text-[9px] uppercase tracking-wider text-[var(--faint)]">Downloaded</p>
              <p className="mt-0.5 font-mono tabular text-[var(--fg)]">
                {formatBytes(download.downloaded)} / {formatBytes(download.totalSize)}
              </p>
            </div>
            <div>
              <p className="text-[9px] uppercase tracking-wider text-[var(--faint)]">ETA</p>
              <p className="mt-0.5 font-mono tabular text-[var(--fg)]">
                {download.state === 'completed' ? 'done' : formatEta(download.etaSeconds)}
              </p>
            </div>
          </div>

          {download.error && (
            <p className="mt-3 flex items-start gap-1.5 rounded-lg bg-[var(--danger-soft)] px-2.5 py-2 text-[11px] text-[var(--danger)]">
              <XCircle className="mt-0.5 size-3.5 shrink-0" />
              {download.error}
            </p>
          )}
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2 border-t border-[var(--border-soft)] px-4 py-2.5">
        {download.streamPath && (
          <>
            {previewable ? (
              <Button
                variant="secondary"
                size="sm"
                onClick={() => navigate(`/player?download=${download.id}`, { state: { from: '/downloads' } })}
              >
                <PreviewIcon className="size-3.5" />
                {previewLabel(kind)}
              </Button>
            ) : (
              <a
                href={download.streamPath}
                target="_blank"
                rel="noreferrer"
                className="btn btn-secondary h-8 px-3 text-xs"
              >
                <Eye className="size-3.5" />
                Open file
              </a>
            )}
            {externalPlayable && (
              <Button variant="ghost" size="sm" loading={busy === 'open'} onClick={openExternal}>
                <Tv className="size-3.5" />
                mpv / VLC
              </Button>
            )}
            <IconButton label="Copy stream URL" size="sm" onClick={() => void copyStream()}>
              <Copy className="size-3.5" />
            </IconButton>
          </>
        )}
        {(download.state === 'downloading' || download.state === 'streaming' || download.state === 'paused') && (
          <Button variant="ghost" size="sm" loading={busy === 'pause' || busy === 'resume'} onClick={pause}>
            {download.state === 'paused' ? <RotateCcw className="size-3.5" /> : <Pause className="size-3.5" />}
            {download.state === 'paused' ? 'Resume' : 'Pause'}
          </Button>
        )}
        <div className="ml-auto flex items-center gap-2">
          <Sparkline values={samples} width={90} height={24} className="hidden opacity-70 sm:block" />
          <IconButton label="Remove from activity" size="sm" onClick={remove}>
            <Trash2 className="size-3.5" />
          </IconButton>
        </div>
      </div>

      <div className="h-[3px] w-full bg-[var(--surface-3)]">
        <motion.div
          className="h-full origin-left"
          style={{
            scaleX: download.progress,
            background:
              toneForState(download.state) === 'success'
                ? 'var(--success)'
                : toneForState(download.state) === 'danger'
                  ? 'var(--danger)'
                  : toneForState(download.state) === 'stream'
                    ? 'var(--badge-stream)'
                    : 'var(--accent)',
          }}
        />
      </div>
    </motion.article>
  )
}

export function DownloadsPage() {
  const downloads = useAppStore(state => state.downloads)
  const [tab, setTab] = useState<Tab>('active')

  const active = downloads.filter(download =>
    ['starting', 'downloading', 'streaming', 'paused'].includes(download.state),
  )
  const completed = downloads.filter(download => download.state === 'completed')
  const failed = downloads.filter(download => download.state === 'failed')
  const totalSpeed = active.reduce((sum, download) => sum + download.speed, 0)
  const visible = tab === 'active' ? active : tab === 'completed' ? completed : failed

  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow="Transfers"
        title="Downloads in flight"
        description="Every torrent is streamed from the engine as pieces arrive. Progress updates four times a second and animates between ticks."
        actions={
          <Link to="/search" className="btn btn-primary">
            <ArrowDownToLine className="size-4" />
            Find torrents
          </Link>
        }
      />

      <div className="grid gap-3 sm:grid-cols-3">
        <div className="panel px-4 py-3">
          <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--muted)]">Down speed</p>
          <p className="mt-1 font-mono text-sm tabular text-[var(--success)]">{formatSpeed(totalSpeed)}</p>
        </div>
        <div className="panel px-4 py-3">
          <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--muted)]">Active</p>
          <p className="mt-1 font-mono text-sm tabular text-[var(--accent)]">{active.length}</p>
        </div>
        <div className="panel px-4 py-3">
          <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--muted)]">Completed</p>
          <p className="mt-1 font-mono text-sm tabular text-[var(--fg)]">{completed.length}</p>
        </div>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-3">
        <SegmentedControl
          options={[
            { value: 'active', label: `Active ${active.length}` },
            { value: 'completed', label: `Completed ${completed.length}` },
            { value: 'failed', label: `Failed ${failed.length}` },
          ]}
          value={tab}
          onChange={value => setTab(value as Tab)}
        />
        <span className="text-[11px] text-[var(--muted)]">
          {active.length ? `${formatSpeed(totalSpeed)} total` : 'Idle'}
        </span>
      </div>

      <SectionHeader
        title={tab === 'active' ? 'Active transfers' : tab === 'completed' ? 'Completed' : 'Failed'}
        hint={
          tab === 'active'
            ? 'Pause, resume, stream inline, or hand the file to mpv.'
            : tab === 'completed'
              ? 'Files stay on disk. Remove the card to tidy the activity list.'
              : 'These tasks stopped before completing. Remove them or start the torrent again.'
        }
      />

      {visible.length === 0 ? (
        <EmptyState
          icon={<ArrowDownToLine className="size-6" />}
          title={tab === 'active' ? 'No active transfers' : tab === 'completed' ? 'Nothing completed yet' : 'No failures'}
          description={
            tab === 'active'
              ? 'Start a download from a torrent page, or stream a single file while it downloads.'
              : 'When transfers finish they move here so you can find them later.'
          }
          action={
            <Link to="/search" className="btn btn-secondary">
              Browse torrents
              <ArrowUpRight className="size-4" />
            </Link>
          }
        />
      ) : (
        <motion.div layout className={cn('grid gap-3', tab === 'active' && 'xl:grid-cols-2')}>
          {visible.map(download => (
            <DownloadCard key={download.id} download={download} />
          ))}
        </motion.div>
      )}
    </div>
  )
}
