import { Link } from 'react-router-dom'
import { ArrowDownToLine, ArrowUpRight, Play, Radio, Server } from 'lucide-react'
import { ProgressBar } from '@/components/ui/Progress'
import { StatusDot } from '@/components/ui/StatusDot'
import { formatBytes, formatEta, formatSpeed } from '@/lib/format'
import { CoverArt } from '@/components/CoverArt'
import { useAppStore } from '@/store/app'

export function ActivityPanel() {
  const downloads = useAppStore(state => state.downloads)
  const player = useAppStore(state => state.player)
  const connected = useAppStore(state => state.connected)
  const bootstrap = useAppStore(state => state.bootstrap)

  const active = downloads
    .filter(download => ['starting', 'downloading', 'streaming', 'paused'].includes(download.state))
    .slice(0, 5)

  return (
    <aside className="sticky top-[88px] hidden h-fit w-[290px] shrink-0 space-y-3 xl:block">
      <section className="panel overflow-hidden">
        <header className="flex items-center justify-between border-b border-[var(--border-soft)] px-4 py-3">
          <div className="flex items-center gap-2">
            <Radio className="size-3.5 text-[var(--accent)]" />
            <h2 className="text-xs font-semibold uppercase tracking-[0.1em] text-[var(--muted-strong)]">Activity</h2>
          </div>
          <Link
            to="/downloads"
            className="inline-flex items-center gap-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--muted)] transition hover:text-[var(--accent)]"
          >
            All
            <ArrowUpRight className="size-3" />
          </Link>
        </header>

        <div className="space-y-1 p-2">
          {active.length === 0 && (
            <div className="px-2 py-7 text-center">
              <ArrowDownToLine className="mx-auto mb-2 size-5 text-[var(--faint)]" />
              <p className="text-xs text-[var(--muted)]">Nothing downloading</p>
              <Link to="/search" className="mt-1 inline-block text-[11px] font-semibold text-[var(--accent)] hover:underline">
                Find something to watch
              </Link>
            </div>
          )}
          {active.map(download => (
            <Link
              key={download.id}
              to="/downloads"
              className="block rounded-[10px] px-2 py-2 transition hover:bg-[var(--hover)]"
            >
              <div className="mb-1.5 flex items-center justify-between gap-2">
                <span className="flex min-w-0 items-center gap-1.5 truncate text-[11px] font-medium text-[var(--fg)]">
                  <CoverArt title={download.torrentName} className="h-8 w-[22px] shrink-0 rounded-[4px]" />
                  <span className="truncate">{download.label}</span>
                </span>
                <span className="shrink-0 font-mono text-[10px] tabular text-[var(--muted)]">
                  {Math.round(download.progress * 100)}%
                </span>
              </div>
              <ProgressBar
                progress={download.progress}
                height={4}
                tone={download.isStreaming ? 'stream' : download.state === 'completed' ? 'success' : 'accent'}
              />
              <div className="mt-1 flex items-center justify-between font-mono text-[9px] tabular text-[var(--faint)]">
                <span>{formatSpeed(download.speed)}</span>
                <span>{formatEta(download.etaSeconds)}</span>
              </div>
            </Link>
          ))}
        </div>
      </section>

      {player.running && (
        <section className="panel overflow-hidden">
          <header className="flex items-center justify-between border-b border-[var(--border-soft)] px-4 py-3">
            <div className="flex items-center gap-2">
              <Play className="size-3.5 text-[var(--badge-stream)]" />
              <h2 className="text-xs font-semibold uppercase tracking-[0.1em] text-[var(--muted-strong)]">Playing</h2>
            </div>
            <StatusDot tone="stream" pulse />
          </header>
          <div className="px-4 py-3">
            <p className="truncate text-xs font-medium text-[var(--fg)]">{player.title ?? 'Untitled stream'}</p>
            <p className="mt-1 text-[10px] text-[var(--muted)]">External player session</p>
            <Link
              to="/player"
              className="mt-3 inline-flex items-center gap-1 text-[11px] font-semibold text-[var(--badge-stream)] hover:underline"
            >
              Open player
              <ArrowUpRight className="size-3" />
            </Link>
          </div>
        </section>
      )}

      <section className="panel overflow-hidden">
        <header className="flex items-center gap-2 border-b border-[var(--border-soft)] px-4 py-3">
          <Server className="size-3.5 text-[var(--muted)]" />
          <h2 className="text-xs font-semibold uppercase tracking-[0.1em] text-[var(--muted-strong)]">Engine</h2>
        </header>
        <dl className="space-y-2 px-4 py-3 text-[11px]">
          <div className="flex items-center justify-between">
            <dt className="text-[var(--muted)]">Connection</dt>
            <dd className="flex items-center gap-1.5 font-semibold text-[var(--fg)]">
              <StatusDot tone={connected ? 'success' : 'danger'} pulse={connected} />
              {connected ? 'Live' : 'Offline'}
            </dd>
          </div>
          <div className="flex items-center justify-between">
            <dt className="text-[var(--muted)]">Downloads</dt>
            <dd className="font-mono tabular text-[var(--fg)]">{downloads.length}</dd>
          </div>
          <div className="flex items-center justify-between">
            <dt className="text-[var(--muted)]">Engine used</dt>
            <dd className="font-mono tabular text-[var(--fg)]">
              {formatBytes(downloads.reduce((sum, download) => sum + download.downloaded, 0))}
            </dd>
          </div>
          {bootstrap && (
            <div className="flex items-center justify-between">
              <dt className="text-[var(--muted)]">Version</dt>
              <dd className="font-mono tabular text-[var(--fg)]">{bootstrap.version}</dd>
            </div>
          )}
        </dl>
      </section>
    </aside>
  )
}
