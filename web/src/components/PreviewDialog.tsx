import { useEffect, useState } from 'react'
import { ArrowRight, Eye, Play, RefreshCw, TriangleAlert } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/Button'
import { Modal } from '@/components/ui/Modal'
import { SkeletonRows } from '@/components/ui/Skeleton'
import { CoverArt } from '@/components/CoverArt'
import { FileTypeIcon, MediaIcon, canPreview, fileKind, isAudio, isImage } from '@/lib/fileIcons'
import { api } from '@/lib/api'
import { formatBytes, pluralize } from '@/lib/format'
import { useAppStore } from '@/store/app'
import type { FileSnapshot, SessionSnapshot } from '@/types/api'

export interface PreviewTarget {
  url: string
  title?: string
  meta?: string
}

interface PreviewDialogProps {
  target: PreviewTarget | null
  onClose: () => void
  onOpen: (session: SessionSnapshot) => void
}

const MAX_VISIBLE_FILES = 120

function actionFor(name: string) {
  const kind = fileKind(name)
  if (!canPreview(kind)) return null
  if (isImage(kind)) return { label: 'View', icon: Eye }
  if (isAudio(kind)) return { label: 'Play', icon: Play }
  return { label: 'Watch', icon: Play }
}

export function PreviewDialog({ target, onClose, onOpen }: PreviewDialogProps) {
  const navigate = useNavigate()
  const setSession = useAppStore(state => state.setSession)
  const addToast = useAppStore(state => state.addToast)
  const [preview, setPreview] = useState<SessionSnapshot | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [attempt, setAttempt] = useState(0)
  const [watching, setWatching] = useState<number | null>(null)

  const targetUrl = target?.url ?? null

  useEffect(() => {
    if (!targetUrl) return
    let cancelled = false
    setPreview(null)
    setError(null)
    setLoading(true)
    setWatching(null)
    api
      .preview(targetUrl)
      .then(session => {
        if (cancelled) return
        setPreview(session)
        setSession(session)
      })
      .catch(reason => {
        if (cancelled) return
        setError(reason instanceof Error ? reason.message : 'Could not load torrent')
      })
      .finally(() => {
        if (!cancelled) setLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [attempt, setSession, targetUrl])

  const watch = async (file: FileSnapshot) => {
    if (watching !== null) return
    setWatching(file.index)
    const from = window.location.pathname + window.location.search
    try {
      const download = await api.watchFile(file.index)
      onClose()
      navigate(`/player?download=${download.id}`, { state: { from } })
    } catch (reason) {
      addToast('error', reason instanceof Error ? reason.message : 'Could not start playback')
      setWatching(null)
    }
  }

  const current = preview && preview.url === targetUrl ? preview : null
  const title = current?.name ?? target?.title ?? 'Torrent preview'
  const coverTitle = current?.name ?? target?.title
  const meta = target?.meta ? `${target.meta} · ` : ''
  const description = current
    ? `${meta}${pluralize(current.files.length, 'file')} · ${formatBytes(current.totalSize)}`
    : (target?.meta ?? 'Fetching torrent metadata and file list…')
  const files = current?.files ?? []
  const visibleFiles = files.slice(0, MAX_VISIBLE_FILES)

  return (
    <Modal
      open={target !== null}
      onOpenChange={open => {
        if (!open) onClose()
      }}
      title={title}
      description={description}
      maxWidth={640}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            Close
          </Button>
          <Button
            variant="primary"
            loading={loading}
            disabled={!current || watching !== null}
            onClick={() => current && onOpen(current)}
          >
            Open torrent
            <ArrowRight className="size-4" />
          </Button>
        </>
      }
    >
      <div className="flex gap-4">
        {coverTitle ? (
          <CoverArt title={coverTitle} className="h-[150px] w-[100px] shrink-0" eager />
        ) : (
          <div className="grid h-[150px] w-[100px] shrink-0 place-items-center rounded-[7px] border border-[var(--border)] bg-[var(--surface-2)]">
            <MediaIcon name="torrent" className="size-10 opacity-70" />
          </div>
        )}
        <div className="min-w-0 flex-1 space-y-3">
          <div>
            <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-[var(--muted)]">
              {loading ? 'Reading torrent' : error ? 'Preview unavailable' : 'Ready to watch'}
            </p>
            <p className="mt-1 break-words text-sm font-medium leading-snug text-[var(--fg)]">
              {title}
            </p>
          </div>
          {current && (
            <div className="flex flex-wrap gap-2 text-[11px] text-[var(--muted)]">
              <span className="rounded-full border border-[var(--border-soft)] bg-[var(--surface-2)] px-2.5 py-1">
                {pluralize(current.files.length, 'file')}
              </span>
              <span className="rounded-full border border-[var(--border-soft)] bg-[var(--surface-2)] px-2.5 py-1">
                {formatBytes(current.totalSize)}
              </span>
            </div>
          )}
          {!current && !error && (
            <div className="space-y-2" aria-hidden>
              <div className="shimmer h-2.5 w-32 rounded-full" />
              <div className="shimmer h-2.5 w-24 rounded-full" />
            </div>
          )}
          {error && (
            <div className="flex items-start gap-2 rounded-lg border border-[color-mix(in_srgb,var(--danger)_24%,var(--border-soft))] bg-[color-mix(in_srgb,var(--danger)_7%,transparent)] px-3 py-2 text-xs text-[var(--danger)]">
              <TriangleAlert className="mt-0.5 size-3.5 shrink-0" />
              <span>{error}</span>
            </div>
          )}
        </div>
      </div>

      <section className="mt-5 border-t border-[var(--border-soft)] pt-4" aria-busy={loading}>
        <div className="mb-2 flex items-center justify-between gap-3">
          <h3 className="text-[10px] font-semibold uppercase tracking-[0.14em] text-[var(--muted)]">
            Files
          </h3>
          {current && (
            <span className="hidden text-[10px] text-[var(--faint)] sm:inline">
              Watch an episode now or open the torrent to choose downloads
            </span>
          )}
        </div>

        {loading && <SkeletonRows count={5} />}

        {!loading && error && (
          <div className="flex items-center justify-between gap-3 rounded-lg border border-[var(--border-soft)] bg-[var(--surface-2)] px-3 py-3">
            <p className="text-xs text-[var(--muted)]">The file list could not be loaded.</p>
            <Button variant="secondary" size="sm" onClick={() => setAttempt(value => value + 1)}>
              <RefreshCw className="size-3.5" />
              Try again
            </Button>
          </div>
        )}

        {!loading && !error && visibleFiles.length === 0 && (
          <p className="rounded-lg border border-[var(--border-soft)] bg-[var(--surface-2)] px-3 py-3 text-xs text-[var(--muted)]">
            This torrent did not report any files.
          </p>
        )}

        {!loading && !error && visibleFiles.length > 0 && (
          <>
            <ul className="divide-y divide-[var(--border-soft)] overflow-hidden rounded-lg border border-[var(--border-soft)] bg-[var(--surface-2)]">
              {visibleFiles.map(file => {
                const action = actionFor(file.name)
                const ActionIcon = action?.icon
                return (
                  <li key={file.index} className="flex items-center gap-2.5 px-3 py-2.5">
                    <FileTypeIcon name={file.name} className="size-4" />
                    <span title={file.name} className="min-w-0 flex-1 truncate text-xs text-[var(--fg)]">
                      {file.name}
                    </span>
                    <span className="shrink-0 text-[10px] tabular-nums text-[var(--muted)]">
                      {formatBytes(file.size)}
                    </span>
                    {action && ActionIcon && (
                      <Button
                        variant="secondary"
                        size="sm"
                        loading={watching === file.index}
                        disabled={watching !== null && watching !== file.index}
                        aria-label={`${action.label} ${file.name}`}
                        onClick={() => void watch(file)}
                        className="h-7 shrink-0 gap-1 px-2 text-[10px]"
                      >
                        <ActionIcon className="size-3" />
                        {action.label}
                      </Button>
                    )}
                  </li>
                )
              })}
            </ul>
            {files.length > visibleFiles.length && (
              <p className="mt-2 text-[10px] text-[var(--faint)]">
                Showing {visibleFiles.length} of {files.length} files. Open the torrent to see the full tree.
              </p>
            )}
          </>
        )}
      </section>
    </Modal>
  )
}
