import { useMemo, useRef, useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { motion } from 'motion/react'
import {
  Copy,
  Download,
  FolderOpen,
  HardDrive,
  LibraryBig,
  Loader2,
  Music,
  Play,
  RefreshCw,
  Search,
  Tv,
  X,
} from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button, IconButton } from '@/components/ui/Button'
import { EmptyState } from '@/components/ui/EmptyState'
import { SkeletonRows } from '@/components/ui/Skeleton'
import { PageHeader } from '@/components/PageHeader'
import { api } from '@/lib/api'
import { useHlsPlayback } from '@/features/player/useHlsPlayback'
import { formatBytes, formatRelative } from '@/lib/format'
import { springs } from '@/lib/motion'
import { useAppStore } from '@/store/app'
import type { LibraryEntry } from '@/types/api'
import { cn } from '@/lib/cn'

type RootFilter = 'all' | 'cache' | 'downloads'

const ROOT_LABEL = { cache: 'Cache', downloads: 'Downloads' } as const

function inlinePlayable(entry: LibraryEntry): boolean {
  return entry.media.playback === 'direct'
}

function streamPathFor(entry: LibraryEntry): string {
  return entry.media.streamPath ?? entry.streamPath
}

function LibraryVideo({ entry }: { entry: LibraryEntry }) {
  const ref = useRef<HTMLVideoElement | null>(null)
  const source = streamPathFor(entry)
  const { attempt, hlsError, isHls, retry } = useHlsPlayback(source, ref)

  return (
    <div className="relative">
      <video
        key={`${source}:${attempt}`}
        ref={ref}
        controls
        autoPlay
        playsInline
        src={isHls ? undefined : source}
        className="aspect-video max-h-[62vh] w-full bg-black object-contain"
      />
      {hlsError && (
        <div className="absolute inset-0 flex flex-col items-center justify-center gap-2 bg-black/85 px-6 text-center text-white">
          <p className="text-sm font-semibold">Could not start the local transcoder</p>
          <p className="max-w-md text-xs text-white/60">
            Make sure ffmpeg is available, then try again.
          </p>
          <button
            type="button"
            onClick={retry}
            className="rounded-[2px] border border-white/20 bg-white/10 px-3 py-1 text-xs"
          >
            Retry
          </button>
        </div>
      )}
    </div>
  )
}

function LibraryRow({
  entry,
  selected,
  onSelect,
}: {
  entry: LibraryEntry
  selected: boolean
  onSelect: () => void
}) {
  const inline = inlinePlayable(entry)
  return (
    <motion.article
      layout
      transition={springs.smooth}
      className={cn(
        'panel overflow-hidden transition-colors',
        selected && 'border-[color-mix(in_srgb,var(--accent)_45%,var(--border-soft))]',
      )}
    >
      <div className="flex flex-wrap items-center gap-3 p-3.5 sm:flex-nowrap">
        <button
          type="button"
          onClick={onSelect}
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
          <Button variant="secondary" size="sm" onClick={onSelect}>
            <Play className="size-3.5" />
            {inline ? 'Play' : 'Preview'}
          </Button>
        </div>
      </div>
    </motion.article>
  )
}

function FilmIcon({ entry }: { entry: LibraryEntry }) {
  return entry.media.playback === 'direct' ? <Play className="size-4" /> : <Tv className="size-4" />
}

function LibraryPlayer({
  entry,
  onClose,
  onCopy,
}: {
  entry: LibraryEntry
  onClose: () => void
  onCopy: () => void
}) {
  const inline = inlinePlayable(entry)
  return (
    <motion.section
      layout
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      className="panel overflow-hidden"
    >
      <div className="flex items-start justify-between gap-3 border-b border-[var(--border-soft)] px-4 py-3">
        <div className="min-w-0">
          <h2 className="truncate text-sm font-semibold text-[var(--fg)]">{entry.name}</h2>
          <p className="mt-0.5 truncate font-mono text-[10px] text-[var(--muted)]">
            {entry.relativePath} · {formatBytes(entry.size)}
          </p>
        </div>
        <div className="flex shrink-0 items-center gap-1.5">
          <IconButton label="Copy stream URL" size="sm" onClick={onCopy}>
            <Copy className="size-3.5" />
          </IconButton>
          <IconButton label="Close player" size="sm" onClick={onClose}>
            <X className="size-4" />
          </IconButton>
        </div>
      </div>

      <div className="bg-black">
        {inline && entry.media.kind === 'audio' && (
          <div className="flex min-h-28 items-center justify-center p-6">
            <audio controls autoPlay src={streamPathFor(entry)} className="w-full max-w-xl" />
          </div>
        )}
        {entry.media.kind === 'video' && <LibraryVideo entry={entry} />}
      </div>

      <div className="flex flex-wrap items-center gap-2 border-t border-[var(--border-soft)] px-4 py-2.5">
        <Badge tone={entry.root === 'cache' ? 'folder' : 'accent'}>{ROOT_LABEL[entry.root]}</Badge>
        <a
          href={streamPathFor(entry)}
          target="_blank"
          rel="noreferrer"
          className="text-[11px] font-semibold text-[var(--accent)] hover:underline"
        >
          Open stream in browser
        </a>
        <span className="ml-auto text-[10px] text-[var(--faint)]">
          {entry.modifiedMs ? `Modified ${formatRelative(entry.modifiedMs / 1000)}` : 'Local file'}
        </span>
      </div>
    </motion.section>
  )
}

export function LibraryPage() {
  const addToast = useAppStore(state => state.addToast)
  const queryClient = useQueryClient()
  const [query, setQuery] = useState('')
  const [root, setRoot] = useState<RootFilter>('all')
  const [selected, setSelected] = useState<LibraryEntry | null>(null)
  const [scanning, setScanning] = useState(false)
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
      if (selected && !data.some(entry => entry.id === selected.id)) setSelected(null)
      addToast('success', `Library refreshed · ${data.length} files`)
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not scan the library')
    } finally {
      setScanning(false)
    }
  }

  const copyStream = async (entry: LibraryEntry) => {
    try {
      await navigator.clipboard.writeText(`${window.location.origin}${streamPathFor(entry)}`)
      addToast('success', 'Stream URL copied')
    } catch {
      addToast('warning', 'Clipboard is unavailable')
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

      {selected && (
        <LibraryPlayer
          entry={selected}
          onClose={() => setSelected(null)}
          onCopy={() => void copyStream(selected)}
        />
      )}

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
              selected={selected?.id === entry.id}
              onSelect={() => setSelected(entry)}
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
    </div>
  )
}
