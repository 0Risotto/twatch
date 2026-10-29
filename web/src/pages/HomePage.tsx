import { useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link, useNavigate } from 'react-router-dom'
import { motion } from 'motion/react'
import {
  ArrowRight,
  ArrowUpRight,
  Clock3,
  Download,
  Link2,
  Search,
  Sparkles,
  Zap,
} from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { EmptyState } from '@/components/ui/EmptyState'
import { SkeletonRows } from '@/components/ui/Skeleton'
import { PageHeader, SectionHeader } from '@/components/PageHeader'
import { PreviewDialog, type PreviewTarget } from '@/components/PreviewDialog'
import { api } from '@/lib/api'
import { displayName, formatBytes, formatRelative, isTorrentInput, pluralize } from '@/lib/format'
import { CoverArt } from '@/components/CoverArt'
import { riseItem, stagger } from '@/lib/motion'
import { useAppStore } from '@/store/app'

export function HomePage() {
  const navigate = useNavigate()
  const session = useAppStore(state => state.session)
  const addToast = useAppStore(state => state.addToast)

  const [url, setUrl] = useState('')
  const [previewTarget, setPreviewTarget] = useState<PreviewTarget | null>(null)

  const history = useQuery({ queryKey: ['history'], queryFn: api.history })

  const recent = useMemo(() => (history.data ?? []).slice(-6).reverse(), [history.data])

  const addTorrent = async (value: string) => {
    const trimmed = value.trim()
    if (!trimmed) {
      addToast('info', 'Paste a magnet or type a title to search')
      return
    }
    if (!isTorrentInput(trimmed)) {
      // A title is a search, not a torrent URL. This keeps the preview button
      // useful even when the user types "Dune" instead of pasting a magnet.
      setUrl('')
      addToast('info', `Searching for “${trimmed}”`)
      navigate(`/search?q=${encodeURIComponent(trimmed)}`)
      return
    }
    setUrl('')
    setPreviewTarget({ url: trimmed })
  }

  const openHistory = (historyUrl: string, title: string) => {
    setPreviewTarget({ url: historyUrl, title })
  }

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Local torrent console"
        title="Torrents, in motion."
        description="Paste a magnet, preview the file tree, and stream while it downloads. The engine runs locally in Rust."
        actions={
          <Link to="/search" className="btn btn-secondary">
            <Search className="size-4" />
            Search torrents
          </Link>
        }
      />

      <motion.section
        variants={stagger}
        initial="initial"
        animate="animate"
        className="relative overflow-hidden rounded-[var(--radius-xl)] border border-[var(--border-soft)] bg-[color-mix(in_srgb,var(--surface)_68%,transparent)] p-5 shadow-[var(--shadow-md)] sm:p-7"
      >

        <div className="relative">
          <div className="mb-4 flex items-center gap-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--muted)]">
            <Sparkles className="size-3.5 text-[var(--accent)]" />
            Start with a magnet or .torrent URL
          </div>
          <form
            onSubmit={event => {
              event.preventDefault()
              void addTorrent(url)
            }}
            className="flex flex-col gap-2 sm:flex-row"
          >
            <div className="relative flex-1">
              <Link2 className="pointer-events-none absolute left-4 top-1/2 size-4 -translate-y-1/2 text-[var(--muted)]" />
              <input
                value={url}
                onChange={event => setUrl(event.target.value)}
                placeholder="Paste a magnet or type a movie / anime title…"
                aria-label="Magnet link or torrent URL"
                autoFocus
                className="h-14 w-full rounded-[14px] border border-[var(--border-soft)] bg-[color-mix(in_srgb,var(--bg)_52%,transparent)] pl-11 pr-4 font-mono text-[13px] text-[var(--fg)] outline-none transition placeholder:text-[var(--faint)] focus:border-[color-mix(in_srgb,var(--accent)_52%,transparent)] focus:shadow-[0_0_0_4px_var(--accent-soft)]"
              />
            </div>
            <Button
              type="submit"
              variant="primary"
              size="lg"
              className="h-14 rounded-[14px] px-6 text-[15px]"
            >
              Preview files
              <ArrowRight className="size-4" />
            </Button>
          </form>
          <div className="mt-3 flex flex-wrap items-center gap-x-5 gap-y-2 text-[11px] text-[var(--muted)]">
            <span className="flex items-center gap-1.5">
              <Zap className="size-3 text-[var(--accent)]" />
              Paste a magnet or type a title to search
            </span>
            <span className="flex items-center gap-1.5">
              <Download className="size-3 text-[var(--success)]" />
              Files stay on your machine
            </span>
            <span className="flex items-center gap-1.5">
              <Clock3 className="size-3 text-[var(--warning)]" />
              Resume from history
            </span>
          </div>

          {session && (
            <div className="mt-5 flex items-center justify-between gap-3 rounded-xl border border-[var(--border-soft)] bg-[var(--surface-2)] px-3.5 py-2.5">
              <div className="min-w-0">
                <p className="truncate text-xs font-medium text-[var(--fg)]">{session.name}</p>
                <p className="text-[10px] text-[var(--muted)]">
                  {pluralize(session.files.length, 'file')} · {formatBytes(session.totalSize)} ·{' '}
                  {session.selectedCount} selected
                </p>
              </div>
              <Link to="/torrent" className="btn btn-secondary h-8 px-3 text-xs">
                Resume
                <ArrowUpRight className="size-3" />
              </Link>
            </div>
          )}
        </div>
      </motion.section>

      <motion.section variants={stagger} initial="initial" animate="animate">
        <SectionHeader
          title="Recent history"
          hint="Reopen a torrent without pasting the link again"
          action={
            <Link to="/history" className="text-[11px] font-semibold text-[var(--accent)] hover:underline">
              Full history
            </Link>
          }
        />
        {history.isLoading ? (
          <SkeletonRows count={3} />
        ) : recent.length === 0 ? (
          <EmptyState
            icon={<Clock3 className="size-6" />}
            title="History is empty"
            description="Every torrent you preview is stored locally. It will appear here for one-click reopening."
          />
        ) : (
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
            {recent.map(entry => (
              <motion.button
                key={entry.url}
                variants={riseItem}
                type="button"
                onClick={() => openHistory(entry.url, displayName(entry))}
                className="panel panel-hover group min-w-0 overflow-hidden p-3 text-left"
              >
                <div className="flex gap-3">
                  <CoverArt title={displayName(entry)} className="h-[86px] w-[58px] shrink-0" />
                  <div className="min-w-0 flex-1">
                    <div className="mb-1 flex justify-end">
                      <ArrowUpRight className="size-4 text-[var(--faint)] transition group-hover:-translate-y-0.5 group-hover:translate-x-0.5 group-hover:text-[var(--accent)]" />
                    </div>
                    <p className="line-clamp-2 text-sm font-medium leading-snug text-[var(--fg)]">
                      {displayName(entry)}
                    </p>
                    <div className="mt-2 flex flex-wrap items-center gap-2 text-[10px] text-[var(--muted)]">
                  <span>{formatRelative(entry.addedAt)}</span>
                  {entry.watchedFiles.length > 0 && (
                    <span className="text-[var(--badge-stream)]">{entry.watchedFiles.length} watched</span>
                  )}
                      {entry.downloadedFiles.length > 0 && (
                        <span className="text-[var(--success)]">{entry.downloadedFiles.length} saved</span>
                      )}
                    </div>
                  </div>
                </div>
              </motion.button>
            ))}
          </div>
        )}
      </motion.section>

      <PreviewDialog
        target={previewTarget}
        onClose={() => setPreviewTarget(null)}
        onOpen={() => {
          setPreviewTarget(null)
          navigate('/torrent')
        }}
      />
    </div>
  )
}
