import { useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'

import { useQuery, useQueryClient } from '@tanstack/react-query'
import {
  Download,
  FolderOpen,
  HardDrive,
  LibraryBig,
  Loader2,
  Music,
  Play,
  RefreshCw,
  Search,
  Trash2,
  Tv,
} from 'lucide-react'
import { motion } from 'motion/react'

import type { LibraryEntry } from '@/types/api'

import { api } from '@/lib/api'
import { cn } from '@/lib/cn'
import { formatBytes, formatRelative } from '@/lib/format'
import { springs } from '@/lib/motion'

import { useAppStore } from '@/store/app'

import { Badge } from '@/components/ui/Badge'
import { Button, IconButton } from '@/components/ui/Button'
import { EmptyState } from '@/components/ui/EmptyState'
import { Modal } from '@/components/ui/Modal'
import { SkeletonRows } from '@/components/ui/Skeleton'

import { PageHeader } from '@/components/PageHeader'

import { usePlayback } from '@/features/player/playbackContext'

type RootFilter = 'all' | 'cache' | 'downloads'

const ROOT_LABEL = { cache: 'Cache', downloads: 'Downloads' } as const

function LibraryRow({
  entry,
  onPlay,
  onDelete,
}: {
  entry: LibraryEntry
  onPlay: () => void
  onDelete: () => void
}) {
  return (
    <motion.article layout transition={springs.smooth} className="panel overflow-hidden">
      <div className="flex flex-wrap items-center gap-3 p-3.5 sm:flex-nowrap">
        <button
          type="button"
          onClick={onPlay}
          className="flex min-w-0 flex-1 items-center gap-3 text-left"
        >
          <span className="grid size-10 shrink-0 place-items-center rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface-2)] text-[var(--accent)]">
            {entry.media.kind === 'audio' ? (
              <Music className="size-4" />
            ) : (
              <FilmIcon entry={entry} />
            )}
          </span>
          <span className="min-w-0">
            <span className="block truncate text-sm font-semibold text-[var(--fg)]">
              {entry.name}
            </span>
            <span className="mt-0.5 block truncate font-mono text-[10px] text-[var(--muted)]">
              {entry.relativePath}
            </span>
          </span>
        </button>

        <div className="flex shrink-0 items-center gap-2 pl-[52px] sm:pl-0">
          <Badge tone={entry.root === 'cache' ? 'folder' : 'accent'}>
            {entry.root === 'cache' ? (
              <HardDrive className="size-3" />
            ) : (
              <Download className="size-3" />
            )}
            {ROOT_LABEL[entry.root]}
          </Badge>
          <span className="hidden font-mono text-[11px] tabular text-[var(--muted)] sm:inline">
            {formatBytes(entry.size)}
          </span>
          <span className="hidden w-20 text-right text-[11px] text-[var(--faint)] md:inline">
            {entry.modifiedMs ? formatRelative(entry.modifiedMs / 1000) : '—'}
          </span>
        </div>

        <div className="ml-auto flex shrink-0 items-center gap-1.5">
          <Button variant="secondary" size="sm" onClick={onPlay}>
            <Play className="size-3.5" />
            Play
          </Button>
          <IconButton label={`Delete ${entry.name}`} variant="danger" size="sm" onClick={onDelete}>
            <Trash2 className="size-3.5" />
          </IconButton>
        </div>
      </div>
    </motion.article>
  )
}

function FilmIcon({ entry }: { entry: LibraryEntry }) {
  return entry.media.playback === 'direct' ? <Play className="size-4" /> : <Tv className="size-4" />
}

export function LibraryPage() {
  const navigate = useNavigate()
  const addToast = useAppStore(state => state.addToast)
  const { track, deactivate } = usePlayback()
  const queryClient = useQueryClient()
  const [query, setQuery] = useState('')
  const [root, setRoot] = useState<RootFilter>('all')
  const [scanning, setScanning] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<LibraryEntry | null>(null)
  const [deleting, setDeleting] = useState(false)
  const library = useQuery({ queryKey: ['library'], queryFn: api.library, staleTime: 30_000 })

  const entries = useMemo(() => library.data ?? [], [library.data])
  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase()
    return entries.filter(entry => {
      if (root !== 'all' && entry.root !== root) return false
      if (!needle) return true
      return (
        entry.name.toLowerCase().includes(needle) ||
        entry.relativePath.toLowerCase().includes(needle)
      )
    })
  }, [entries, query, root])

  const totalSize = entries.reduce((sum, entry) => sum + entry.size, 0)
  const cacheCount = entries.filter(entry => entry.root === 'cache').length

  const rescan = async () => {
    if (scanning) return
    setScanning(true)
    try {
      const data = await api.rescanLibrary()
      queryClient.setQueryData(['library'], data)
      addToast('success', `Library refreshed · ${data.length} files`)
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not scan the library')
    } finally {
      setScanning(false)
    }
  }

  const remove = async () => {
    if (!deleteTarget) return
    setDeleting(true)
    try {
      const report = await api.deleteLibraryFile(deleteTarget.id)
      queryClient.setQueryData<LibraryEntry[]>(['library'], current =>
        (current ?? []).filter(entry => entry.id !== deleteTarget.id),
      )
      if (track?.source === 'library' && track.library?.id === deleteTarget.id) deactivate()
      setDeleteTarget(null)
      if (report.failed.length > 0) {
        addToast('warning', `Deleted ${report.deleted}; ${report.failed.length} failed`)
      } else {
        addToast('success', 'File deleted')
      }
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not delete that file')
    } finally {
      setDeleting(false)
    }
  }

  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow="On this machine"
        title="Local library"
        description="Media already in the twatch cache and completed downloads folder. Files are indexed locally and streamed with range support."
        actions={
          <Button variant="secondary" loading={scanning} onClick={() => void rescan()}>
            <RefreshCw className={cn('size-4', scanning && 'animate-spin')} />
            Rescan
          </Button>
        }
      />

      <div className="grid gap-3 sm:grid-cols-3">
        <div className="panel px-4 py-3">
          <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--muted)]">
            Files
          </p>
          <p className="mt-1 font-mono text-sm tabular text-[var(--fg)]">{entries.length}</p>
        </div>
        <div className="panel px-4 py-3">
          <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--muted)]">
            Cache
          </p>
          <p className="mt-1 font-mono text-sm tabular text-[var(--folder)]">{cacheCount}</p>
        </div>
        <div className="panel px-4 py-3">
          <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--muted)]">
            Total size
          </p>
          <p className="mt-1 font-mono text-sm tabular text-[var(--fg)]">
            {formatBytes(totalSize)}
          </p>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <div className="relative min-w-[220px] flex-1">
          <Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-[var(--muted)]" />
          <input
            value={query}
            onChange={event => setQuery(event.target.value)}
            placeholder="Filter by title or folder…"
            aria-label="Filter local library"
            className="h-9 w-full rounded-[4px] border border-[var(--border-soft)] bg-[var(--surface)] pl-9 pr-3 text-xs text-[var(--fg)] outline-none placeholder:text-[var(--faint)] focus:border-[var(--accent)]"
          />
        </div>
        <div className="flex items-center gap-0.5 rounded-[3px] border border-[var(--border-soft)] bg-[var(--surface-2)] p-0.5">
          {(['all', 'cache', 'downloads'] as const).map(value => (
            <button
              key={value}
              type="button"
              onClick={() => setRoot(value)}
              className={cn(
                'h-7 rounded-[2px] px-3 text-xs font-semibold transition-colors',
                root === value
                  ? 'bg-[var(--surface)] text-[var(--fg)] shadow-[var(--shadow-sm)]'
                  : 'text-[var(--muted)] hover:text-[var(--fg)]',
              )}
            >
              {value === 'all' ? 'All' : ROOT_LABEL[value]}
            </button>
          ))}
        </div>
        <span className="text-[11px] text-[var(--muted)]">{filtered.length} shown</span>
      </div>

      {library.isLoading && <SkeletonRows count={5} />}

      {library.isError && (
        <EmptyState
          icon={<LibraryBig className="size-6" />}
          title="Could not read the library"
          description={library.error instanceof Error ? library.error.message : 'Try rescanning.'}
          action={
            <Button variant="secondary" onClick={() => void rescan()}>
              Rescan
            </Button>
          }
        />
      )}

      {!library.isLoading && !library.isError && entries.length === 0 && (
        <EmptyState
          icon={<FolderOpen className="size-6" />}
          title="No local media yet"
          description="Completed downloads and files in the twatch cache appear here automatically."
        />
      )}

      {!library.isLoading && !library.isError && entries.length > 0 && filtered.length === 0 && (
        <EmptyState
          icon={<Search className="size-6" />}
          title="Nothing matches that filter"
          description="Try a different title or switch roots."
        />
      )}

      {!library.isLoading && !library.isError && filtered.length > 0 && (
        <div className="space-y-2">
          {filtered.map(entry => (
            <LibraryRow
              key={entry.id}
              entry={entry}
              onPlay={() =>
                navigate(`/player?library=${encodeURIComponent(entry.id)}`, {
                  state: { from: '/library' },
                })
              }
              onDelete={() => setDeleteTarget(entry)}
            />
          ))}
        </div>
      )}

      {scanning && (
        <p className="flex items-center gap-2 text-[11px] text-[var(--muted)]">
          <Loader2 className="size-3.5 animate-spin" />
          Scanning the cache and downloads folder…
        </p>
      )}

      <Modal
        open={deleteTarget !== null}
        onOpenChange={open => {
          if (!open) setDeleteTarget(null)
        }}
        title="Delete this file?"
        description="This removes the file from disk. It cannot be undone."
        footer={
          <>
            <Button variant="ghost" onClick={() => setDeleteTarget(null)}>
              Cancel
            </Button>
            <Button variant="danger" loading={deleting} onClick={() => void remove()}>
              <Trash2 className="size-4" />
              Delete file
            </Button>
          </>
        }
      >
        <p className="break-all font-mono text-xs text-[var(--fg)]">{deleteTarget?.relativePath}</p>
      </Modal>
    </div>
  )
}
