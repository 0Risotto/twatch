import { useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useNavigate } from 'react-router-dom'
import { motion } from 'motion/react'
import { ArrowUpRight, Clock3, Download, Pencil, Play, Search, Trash2 } from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button, IconButton } from '@/components/ui/Button'
import { EmptyState } from '@/components/ui/EmptyState'
import { Modal } from '@/components/ui/Modal'
import { ProgressBar } from '@/components/ui/Progress'
import { SkeletonRows } from '@/components/ui/Skeleton'
import { PageHeader } from '@/components/PageHeader'
import { api } from '@/lib/api'
import { displayName, formatDate, formatRelative, formatTime } from '@/lib/format'
import { CoverArt } from '@/components/CoverArt'
import { riseItem, stagger } from '@/lib/motion'
import { useAppStore } from '@/store/app'
import type { HistoryEntry, PlaybackProgress } from '@/types/api'

interface ResumePoint {
  name: string
  progress: PlaybackProgress
}

function resumePoint(entry: HistoryEntry): ResumePoint | null {
  const points = Object.entries(entry.progress ?? {})
  if (points.length === 0) return null
  const [name, progress] = points.sort(([, left], [, right]) => right.updatedAt - left.updatedAt)[0]
  return { name, progress }
}

export function HistoryPage() {
  const navigate = useNavigate()
  const addToast = useAppStore(state => state.addToast)
  const setSession = useAppStore(state => state.setSession)

  const history = useQuery({ queryKey: ['history'], queryFn: api.history })
  const [query, setQuery] = useState('')
  const [renameTarget, setRenameTarget] = useState<HistoryEntry | null>(null)
  const [renameValue, setRenameValue] = useState('')
  const [deleteTarget, setDeleteTarget] = useState<HistoryEntry | null>(null)
  const [busy, setBusy] = useState<string | null>(null)

  const entries = useMemo(() => {
    const all = [...(history.data ?? [])].reverse()
    const needle = query.trim().toLowerCase()
    if (!needle) return all
    return all.filter(entry =>
      `${displayName(entry)} ${entry.torrentName} ${entry.url}`.toLowerCase().includes(needle),
    )
  }, [history.data, query])

  const open = async (entry: HistoryEntry) => {
    setBusy(`open:${entry.url}`)
    try {
      const session = await api.openHistory(entry.url)
      setSession(session)
      navigate('/torrent')
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not open torrent')
    } finally {
      setBusy(null)
    }
  }

  const resume = async (entry: HistoryEntry) => {
    const point = resumePoint(entry)
    if (!point) return
    setBusy(`resume:${entry.url}`)
    try {
      const session = await api.openHistory(entry.url)
      setSession(session)
      const file = session.files.find(item => item.name === point.name)
      if (!file) {
        addToast('warning', 'That file is no longer in the torrent')
        navigate('/torrent')
        return
      }
      const download = await api.watchFile(file.index)
      navigate(`/player?download=${download.id}`)
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not resume playback')
    } finally {
      setBusy(null)
    }
  }

  const saveRename = async () => {
    if (!renameTarget || !renameValue.trim()) return
    setBusy('rename')
    try {
      await api.renameHistory(renameTarget.url, renameValue.trim())
      await history.refetch()
      setRenameTarget(null)
      addToast('success', 'History entry renamed')
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not rename entry')
    } finally {
      setBusy(null)
    }
  }

  const remove = async () => {
    if (!deleteTarget) return
    setBusy('delete')
    try {
      await api.deleteHistory(deleteTarget.url)
      await history.refetch()
      setDeleteTarget(null)
      addToast('success', 'Removed from history')
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not delete entry')
    } finally {
      setBusy(null)
    }
  }

  return (
    <div className="space-y-5">
      <PageHeader
        eyebrow="Local memory"
        title="Everything you have opened"
        description="History is a local JSON file under your config directory. Reopen a torrent without saving the magnet again."
        actions={
          <Button variant="primary" size="sm" onClick={() => navigate('/')}>
            <Download className="size-4" />
            Add torrent
          </Button>
        }
      />

      <div className="panel p-3">
        <div className="relative">
          <Search className="pointer-events-none absolute left-3.5 top-1/2 size-4 -translate-y-1/2 text-[var(--muted)]" />
          <input
            value={query}
            onChange={event => setQuery(event.target.value)}
            placeholder="Filter by name or URL…"
            aria-label="Filter history"
            className="h-11 w-full rounded-[3px] border border-[var(--border-soft)] bg-[var(--surface-2)] pl-10 pr-3 text-sm text-[var(--fg)] outline-none transition placeholder:text-[var(--faint)] focus:border-[var(--accent)]"
          />
        </div>
      </div>

      {history.isLoading && <SkeletonRows count={5} />}

      {!history.isLoading && entries.length === 0 && (
        <EmptyState
          icon={<Clock3 className="size-6" />}
          title={query ? 'No matching entries' : 'No history yet'}
          description={
            query
              ? 'Try a shorter filter. History matches both the display name and the source URL.'
              : 'Preview a torrent and it is recorded here automatically, including watched and downloaded files.'
          }
          action={
            <Button variant="secondary" onClick={() => navigate('/')}>
              Add your first torrent
            </Button>
          }
        />
      )}

      {entries.length > 0 && (
        <motion.div variants={stagger} initial="initial" animate="animate" className="space-y-2">
          {entries.map(entry => {
            const point = resumePoint(entry)
            const fraction = point?.progress.duration
              ? Math.min(1, point.progress.position / point.progress.duration)
              : 0
            return (
              <motion.div
                key={entry.url}
                variants={riseItem}
                className="group flex flex-wrap items-center gap-3 rounded-[4px] border border-[var(--border-soft)] bg-[color-mix(in_srgb,var(--surface)_88%,transparent)] px-3.5 py-3 transition-colors hover:border-[var(--border-strong)]"
              >
                <CoverArt title={displayName(entry)} className="h-16 w-11 shrink-0" />
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="truncate text-[13px] font-medium text-[var(--fg)]">
                      {displayName(entry)}
                    </p>
                    {entry.watchedFiles.length > 0 && (
                      <Badge tone="stream">{entry.watchedFiles.length} watched</Badge>
                    )}
                    {entry.downloadedFiles.length > 0 && (
                      <Badge tone="success">{entry.downloadedFiles.length} saved</Badge>
                    )}
                  </div>
                  <p className="mt-1 truncate font-mono text-[10px] text-[var(--faint)]">
                    {entry.url}
                  </p>
                  {point && (
                    <div className="mt-2 max-w-sm">
                      <div className="flex items-center justify-between gap-2 text-[10px] text-[var(--muted)]">
                        <span className="truncate">{point.name}</span>
                        <span className="shrink-0 font-mono tabular">
                          {formatTime(point.progress.position)} /{' '}
                          {formatTime(point.progress.duration)}
                        </span>
                      </div>
                      <div className="mt-1">
                        <ProgressBar progress={fraction} tone="stream" height={4} />
                      </div>
                    </div>
                  )}
                </div>
                <div className="hidden shrink-0 text-right sm:block">
                  <p className="text-[11px] text-[var(--muted)]">{formatRelative(entry.addedAt)}</p>
                  <p className="text-[10px] text-[var(--faint)]">{formatDate(entry.addedAt)}</p>
                </div>
                <div className="flex items-center gap-1">
                  {point && (
                    <Button
                      variant="primary"
                      size="sm"
                      loading={busy === `resume:${entry.url}`}
                      onClick={() => void resume(entry)}
                    >
                      <Play className="size-3.5" />
                      Resume
                    </Button>
                  )}
                  <Button
                    variant="secondary"
                    size="sm"
                    loading={busy === `open:${entry.url}`}
                    onClick={() => void open(entry)}
                  >
                    Reopen
                    <ArrowUpRight className="size-3.5" />
                  </Button>
                  <IconButton
                    label="Rename history entry"
                    size="sm"
                    onClick={() => {
                      setRenameTarget(entry)
                      setRenameValue(displayName(entry))
                    }}
                  >
                    <Pencil className="size-3.5" />
                  </IconButton>
                  <IconButton
                    label="Delete history entry"
                    size="sm"
                    onClick={() => setDeleteTarget(entry)}
                  >
                    <Trash2 className="size-3.5" />
                  </IconButton>
                </div>
              </motion.div>
            )
          })}
        </motion.div>
      )}

      <Modal
        open={renameTarget !== null}
        onOpenChange={open => {
          if (!open) setRenameTarget(null)
        }}
        title="Rename history entry"
        description="This only changes the display name in twatch. The torrent URL is untouched."
        footer={
          <>
            <Button variant="ghost" onClick={() => setRenameTarget(null)}>
              Cancel
            </Button>
            <Button variant="primary" loading={busy === 'rename'} onClick={() => void saveRename()}>
              Save name
            </Button>
          </>
        }
      >
        <input
          value={renameValue}
          onChange={event => setRenameValue(event.target.value)}
          onKeyDown={event => {
            if (event.key === 'Enter') void saveRename()
          }}
          autoFocus
          className="input"
          aria-label="History display name"
        />
      </Modal>

      <Modal
        open={deleteTarget !== null}
        onOpenChange={open => {
          if (!open) setDeleteTarget(null)
        }}
        title="Remove from history?"
        description="The files on disk are not touched. You can add the torrent again later with its magnet."
        footer={
          <>
            <Button variant="ghost" onClick={() => setDeleteTarget(null)}>
              Cancel
            </Button>
            <Button variant="danger" loading={busy === 'delete'} onClick={() => void remove()}>
              <Trash2 className="size-4" />
              Remove entry
            </Button>
          </>
        }
      >
        <p className="truncate text-sm text-[var(--fg)]">
          {deleteTarget ? displayName(deleteTarget) : ''}
        </p>
      </Modal>
    </div>
  )
}
