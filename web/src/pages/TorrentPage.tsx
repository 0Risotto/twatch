import { useEffect, useMemo, useRef, useState } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import { Navigate, useNavigate } from 'react-router-dom'
import { AnimatePresence, motion } from 'motion/react'
import {
  Check,
  ChevronDown,
  Eye,
  ChevronRight,
  Download,
  Folder,
  FolderOpen,
  Minus,
  Pencil,
  Play,
  Search,
  Trash2,
  X,
} from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button, IconButton } from '@/components/ui/Button'
import { EmptyState } from '@/components/ui/EmptyState'
import { Modal } from '@/components/ui/Modal'
import { PageHeader } from '@/components/PageHeader'
import { api } from '@/lib/api'
import { formatBytes, pluralize } from '@/lib/format'
import { cn } from '@/lib/cn'
import { useAppStore } from '@/store/app'
import type { DownloadSnapshot, FileSnapshot } from '@/types/api'
import { allFolderPaths, buildTreeRows, type FolderInfo, type TreeRow } from '@/features/torrent/tree'
import { canPreview, fileKind, previewLabel, FileTypeIcon } from '@/lib/fileIcons'
import { CoverArt } from '@/components/CoverArt'

function Checkbox({
  checked,
  indeterminate,
  onChange,
  label,
}: {
  checked: boolean
  indeterminate?: boolean
  onChange: () => void
  label: string
}) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={indeterminate ? 'mixed' : checked}
      aria-label={label}
      onClick={event => {
        event.stopPropagation()
        onChange()
      }}
      className={cn(
        'grid size-[18px] shrink-0 place-items-center rounded-[5px] border transition-colors',
        checked || indeterminate
          ? 'border-[var(--accent)] bg-[var(--accent)] text-[var(--accent-fg)]'
          : 'border-[var(--border-strong)] bg-[color-mix(in_srgb,var(--surface)_60%,transparent)] hover:border-[var(--accent)]',
      )}
    >
      {indeterminate ? <Minus className="size-3" strokeWidth={3} /> : checked ? <Check className="size-3" strokeWidth={3} /> : null}
    </button>
  )
}

function FileBadges({ file }: { file: FileSnapshot }) {
  return (
    <span className="flex shrink-0 items-center gap-1">
      {file.downloading && <Badge tone="download" dot>downloading</Badge>}
      {file.watched && <Badge tone="stream">watched</Badge>}
      {file.downloaded && <Badge tone="success">saved</Badge>}
    </span>
  )
}

export function TorrentPage() {
  const session = useAppStore(state => state.session)
  const setSession = useAppStore(state => state.setSession)
  const addToast = useAppStore(state => state.addToast)
  const navigate = useNavigate()

  const [filter, setFilter] = useState('')
  const [expanded, setExpanded] = useState<Set<string>>(new Set())
  const [renameTarget, setRenameTarget] = useState<{ index: number; name: string } | null>(null)
  const [renameValue, setRenameValue] = useState('')
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [busy, setBusy] = useState<string | null>(null)
  const scrollRef = useRef<HTMLDivElement>(null)
  const sessionUrl = session?.url

  useEffect(() => {
    if (!session) return
    setExpanded(new Set(session.files.length <= 60 ? allFolderPaths(session.files) : []))
    setFilter('')
  }, [sessionUrl])

  const rows = useMemo(
    () => (session ? buildTreeRows(session.files, expanded, filter) : []),
    [session, expanded, filter],
  )

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 42,
    overscan: 12,
  })

  if (!session) {
    return <Navigate to="/" replace />
  }

  const selectedIndices = session.files.filter(file => file.selected).map(file => file.index)
  const selectedFiles = session.files.filter(file => file.selected)
  const selectedFile = selectedIndices.length === 1 ? session.files.find(file => file.index === selectedIndices[0]) ?? null : null
  const selectedKind = selectedFile ? fileKind(selectedFile.name) : null
  const canOpenSelected = selectedKind ? canPreview(selectedKind) : false
  const selectedLabel = selectedKind && canPreview(selectedKind) ? previewLabel(selectedKind) : 'Watch'
  const OpenIcon = selectedKind === 'image' ? Eye : Play
  const selectedSize = selectedFiles.reduce((sum, file) => sum + file.size, 0)
  const allSelected = selectedIndices.length === session.files.length && session.files.length > 0
  const someSelected = selectedIndices.length > 0 && !allSelected

  const patchSelection = async (indices: number[], selected: boolean) => {
    const previous = useAppStore.getState().session
    useAppStore.getState().optimisticSelect(indices, selected)
    try {
      const next = await api.selectFiles(indices, selected)
      setSession(next)
    } catch (error) {
      if (previous) setSession(previous)
      addToast('error', error instanceof Error ? error.message : 'Could not update selection')
    }
  }

  const toggleFile = (file: FileSnapshot) => void patchSelection([file.index], !file.selected)

  const toggleFolder = (folder: FolderInfo) => {
    const all = folder.selected === folder.fileIndices.length
    void patchSelection(folder.fileIndices, !all)
  }

  const toggleAll = () => void patchSelection(session.files.map(file => file.index), !allSelected)

  const expandAll = () => setExpanded(new Set(allFolderPaths(session.files)))
  const collapseAll = () => setExpanded(new Set())

  const addDownloadToStore = (download: DownloadSnapshot) => {
    const downloads = useAppStore.getState().downloads
    if (!downloads.some(item => item.id === download.id)) {
      useAppStore.getState().setDownloads([download, ...downloads])
    }
  }

  const watch = async (index: number) => {
    if (busy) return
    setBusy('watch')
    try {
      const download = await api.watchFile(index)
      addDownloadToStore(download)
      sessionStorage.setItem('twatch.pendingStream', String(download.id))
      addToast('info', 'Preparing stream — the player will start as soon as pieces arrive.')
      navigate(`/player?download=${download.id}`, { state: { from: '/torrent' } })
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not start stream')
    } finally {
      setBusy(null)
    }
  }

  const download = async () => {
    if (selectedIndices.length === 0 || busy) return
    setBusy('download')
    try {
      const entry = await api.downloadFiles(selectedIndices)
      addDownloadToStore(entry)
      addToast('success', `Started ${entry.label}`)
      navigate('/downloads')
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not start download')
    } finally {
      setBusy(null)
    }
  }

  const remove = async () => {
    if (selectedIndices.length === 0 || busy) return
    setBusy('delete')
    try {
      const report = await api.deleteFiles(selectedIndices)
      setDeleteOpen(false)
      if (report.failed.length) {
        addToast('warning', `Deleted ${report.deleted}; ${report.failed.length} failed`)
      } else {
        addToast('success', `Deleted ${pluralize(report.deleted, 'file')}`)
      }
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not delete files')
    } finally {
      setBusy(null)
    }
  }

  const saveRename = async () => {
    if (!renameTarget || !renameValue.trim() || busy) return
    setBusy('rename')
    try {
      const next = await api.renameFile(renameTarget.index, renameValue.trim())
      setSession(next)
      setRenameTarget(null)
      addToast('success', 'File renamed')
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not rename file')
    } finally {
      setBusy(null)
    }
  }

  const closeSession = async () => {
    try {
      await api.closeSession()
      setSession(null)
      navigate('/')
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : 'Could not close session')
    }
  }

  return (
    <div className="space-y-4">
      <PageHeader
        eyebrow="Torrent session"
        media={<CoverArt title={session.name} className="h-[112px] w-[74px] shrink-0 sm:h-[126px] sm:w-[84px]" />}
        title={session.name}
        description={`${pluralize(session.files.length, 'file')} · ${formatBytes(session.totalSize)}${session.downloadRoot ? ` · downloads into ${session.downloadRoot}/` : ''}`}
        actions={
          <>
            <Button variant="ghost" size="sm" onClick={() => void closeSession()}>
              Close
            </Button>
            {canOpenSelected && (
              <Button
                variant="secondary"
                size="sm"
                loading={busy === 'watch'}
                onClick={() => selectedFile && void watch(selectedFile.index)}
              >
                <OpenIcon className="size-4" />
                {selectedLabel}
              </Button>
            )}
            <Button
              variant="primary"
              size="sm"
              disabled={selectedIndices.length === 0}
              loading={busy === 'download'}
              onClick={() => void download()}
            >
              <Download className="size-4" />
              Download
            </Button>
          </>
        }
      />

      <div className="grid gap-3 sm:grid-cols-3">
        {[
          ['Files', pluralize(session.files.length, 'file'), 'text-[var(--accent)]'],
          ['Selected', `${selectedIndices.length} · ${formatBytes(selectedSize)}`, 'text-[var(--success)]'],
          ['Total', formatBytes(session.totalSize), 'text-[var(--fg)]'],
        ].map(([label, value, tone]) => (
          <div key={label} className="panel px-4 py-3">
            <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--muted)]">{label}</p>
            <p className={cn('mt-1 font-mono text-sm tabular', tone)}>{value}</p>
          </div>
        ))}
      </div>

      <div className="panel overflow-hidden">
        <div className="flex flex-wrap items-center gap-2 border-b border-[var(--border-soft)] p-3">
          <Checkbox
            checked={allSelected}
            indeterminate={someSelected}
            onChange={toggleAll}
            label="Select all files"
          />
          <div className="relative min-w-[180px] flex-1">
            <Search className="pointer-events-none absolute left-3 top-1/2 size-3.5 -translate-y-1/2 text-[var(--muted)]" />
            <input
              value={filter}
              onChange={event => setFilter(event.target.value)}
              placeholder="Filter files…"
              className="h-9 w-full rounded-[9px] border border-[var(--border-soft)] bg-[var(--surface-2)] pl-9 pr-3 text-xs text-[var(--fg)] outline-none transition placeholder:text-[var(--faint)] focus:border-[var(--accent)]"
            />
          </div>
          <div className="flex items-center gap-1">
            <Button variant="ghost" size="sm" onClick={expandAll}>
              Expand all
            </Button>
            <Button variant="ghost" size="sm" onClick={collapseAll}>
              Collapse
            </Button>
            <Button
              variant="danger"
              size="sm"
              disabled={selectedIndices.length === 0}
              onClick={() => setDeleteOpen(true)}
            >
              <Trash2 className="size-3.5" />
              Delete
            </Button>
          </div>
        </div>

        <div ref={scrollRef} className="relative max-h-[calc(100vh-360px)] min-h-[340px] overflow-auto">
          {rows.length === 0 ? (
            <EmptyState
              icon={<Search className="size-6" />}
              title="No matching files"
              description="Try a different filter, or clear it to see the full torrent."
              className="m-4 border-0 bg-transparent"
            />
          ) : (
            <div style={{ height: virtualizer.getTotalSize(), position: 'relative' }}>
              {virtualizer.getVirtualItems().map(item => {
                const row = rows[item.index]
                return (
                  <div
                    key={row.key}
                    className="absolute left-0 top-0 w-full px-2"
                    style={{ height: item.size, transform: `translateY(${item.start}px)` }}
                  >
                    <TreeRowView
                      row={row}
                      expanded={expanded}
                      onToggleFolder={path => {
                        setExpanded(current => {
                          const next = new Set(current)
                          if (next.has(path)) next.delete(path)
                          else next.add(path)
                          return next
                        })
                      }}
                      onToggleFile={toggleFile}
                      onToggleFolderSelection={toggleFolder}
                      onOpen={index => void watch(index)}
                      onRename={(index, name) => {
                        setRenameTarget({ index, name })
                        setRenameValue(name.split('/').pop() ?? name)
                      }}
                    />
                  </div>
                )
              })}
            </div>
          )}
        </div>

        <AnimatePresence>
          {selectedIndices.length > 0 && (
            <motion.div
              initial={{ y: 18, opacity: 0 }}
              animate={{ y: 0, opacity: 1 }}
              exit={{ y: 18, opacity: 0 }}
              transition={{ type: 'spring', stiffness: 380, damping: 34 }}
              className="flex items-center justify-between gap-3 border-t border-[var(--border-soft)] bg-[var(--surface-2)] px-3 py-2.5"
            >
              <p className="text-xs text-[var(--muted)]">
                <span className="font-semibold text-[var(--fg)]">{selectedIndices.length}</span> selected ·{' '}
                {formatBytes(selectedSize)}
              </p>
              <div className="flex items-center gap-2">
                <Button variant="ghost" size="sm" onClick={() => void patchSelection(selectedIndices, false)}>
                  <X className="size-3.5" />
                  Clear
                </Button>
                {canOpenSelected && (
                  <Button
                    variant="secondary"
                    size="sm"
                    loading={busy === 'watch'}
                    onClick={() => selectedFile && void watch(selectedFile.index)}
                  >
                    <OpenIcon className="size-3.5" />
                    {selectedLabel}
                  </Button>
                )}
                <Button variant="primary" size="sm" loading={busy === 'download'} onClick={() => void download()}>
                  <Download className="size-3.5" />
                  Download
                </Button>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      <Modal
        open={deleteOpen}
        onOpenChange={setDeleteOpen}
        title={`Delete ${pluralize(selectedIndices.length, 'file')}?`}
        description="The files are removed from disk. The torrent entry stays in history so you can fetch it again."
        footer={
          <>
            <Button variant="ghost" onClick={() => setDeleteOpen(false)}>
              Cancel
            </Button>
            <Button variant="danger" loading={busy === 'delete'} onClick={() => void remove()}>
              <Trash2 className="size-4" />
              Delete files
            </Button>
          </>
        }
      >
        <ul className="max-h-52 space-y-1 overflow-y-auto">
          {selectedFiles.slice(0, 20).map(file => (
            <li key={file.index} className="truncate font-mono text-xs text-[var(--muted)]">
              {file.name}
            </li>
          ))}
          {selectedFiles.length > 20 && (
            <li className="text-xs text-[var(--faint)]">…and {selectedFiles.length - 20} more</li>
          )}
        </ul>
      </Modal>

      <Modal
        open={renameTarget !== null}
        onOpenChange={open => {
          if (!open) setRenameTarget(null)
        }}
        title="Rename file"
        description="Names cannot contain path separators. The file on disk is renamed too."
        footer={
          <>
            <Button variant="ghost" onClick={() => setRenameTarget(null)}>
              Cancel
            </Button>
            <Button variant="primary" loading={busy === 'rename'} onClick={() => void saveRename()}>
              Rename
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
          aria-label="New file name"
        />
      </Modal>
    </div>
  )
}

function TreeRowView({
  row,
  expanded,
  onToggleFolder,
  onToggleFile,
  onToggleFolderSelection,
  onOpen,
  onRename,
}: {
  row: TreeRow
  expanded: Set<string>
  onToggleFolder: (path: string) => void
  onToggleFile: (file: FileSnapshot) => void
  onToggleFolderSelection: (folder: FolderInfo) => void
  onOpen: (index: number) => void
  onRename: (index: number, name: string) => void
}) {
  const indent = row.depth * 18

  if (row.type === 'folder' && row.folder) {
    const folder = row.folder
    const isOpen = expanded.has(folder.path)
    const all = folder.selected === folder.fileIndices.length && folder.fileIndices.length > 0
    const some = folder.selected > 0 && !all
    return (
      <div
        className="group flex h-[42px] items-center gap-2 rounded-[9px] px-2 hover:bg-[var(--hover)]"
        style={{ paddingLeft: indent + 8 }}
        onClick={() => onToggleFolder(folder.path)}
        role="treeitem"
        aria-expanded={isOpen}
        tabIndex={0}
      >
        <Checkbox
          checked={all}
          indeterminate={some}
          onChange={() => onToggleFolderSelection(folder)}
          label={`Select folder ${folder.path}`}
        />
        <span className="grid size-6 place-items-center text-[var(--folder)]">
          {isOpen ? <FolderOpen className="size-4" /> : <Folder className="size-4" />}
        </span>
        <span className="min-w-0 flex-1 truncate text-[13px] font-medium text-[var(--fg)]">{row.name}/</span>
        <span className="font-mono text-[10px] tabular text-[var(--faint)]">
          {folder.selected}/{folder.fileIndices.length} · {formatBytes(folder.size)}
        </span>
        <span className="text-[var(--muted)]">
          {isOpen ? <ChevronDown className="size-4" /> : <ChevronRight className="size-4" />}
        </span>
      </div>
    )
  }

  const file = row.file!
  const kind = fileKind(file.name)
  const previewable = canPreview(kind)
  const RowOpenIcon = kind === 'image' ? Eye : Play
  return (
    <div
      className="group flex h-[42px] items-center gap-2 rounded-[9px] px-2 hover:bg-[var(--hover)]"
      style={{ paddingLeft: indent + 30 }}
      onClick={() => onToggleFile(file)}
      role="treeitem"
      tabIndex={0}
    >
      <Checkbox checked={file.selected} onChange={() => onToggleFile(file)} label={`Select ${file.name}`} />
      <span className="grid size-6 place-items-center text-[var(--muted)]">
        <FileTypeIcon name={file.name} className="size-4" />
      </span>
      <span className="min-w-0 flex-1 truncate text-[13px] text-[var(--fg)]">{row.name}</span>
      <FileBadges file={file} />
      <span className="hidden w-20 text-right font-mono text-[10px] tabular text-[var(--faint)] sm:inline">
        {formatBytes(file.size)}
      </span>
      <span className="flex items-center opacity-0 transition-opacity group-hover:opacity-100">
        {previewable && (
          <IconButton
            label={`${previewLabel(kind)} ${row.name}`}
            size="sm"
            onClick={event => {
              event.stopPropagation()
              onOpen(file.index)
            }}
          >
            <RowOpenIcon className="size-3.5" />
          </IconButton>
        )}
        <IconButton
          label={`Rename ${row.name}`}
          size="sm"
          onClick={event => {
            event.stopPropagation()
            onRename(file.index, file.name)
          }}
        >
          <Pencil className="size-3.5" />
        </IconButton>
      </span>
      {!file.selected && <span className="w-[52px] shrink-0 sm:hidden" />}
    </div>
  )
}
