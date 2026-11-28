import {
  Check,
  ChevronDown,
  ChevronRight,
  Eye,
  Folder,
  FolderOpen,
  Minus,
  Pencil,
  Play,
} from 'lucide-react'

import type { FileSnapshot } from '@/types/api'

import { cn } from '@/lib/cn'
import { canPreview, fileKind, FileTypeIcon, previewLabel } from '@/lib/fileIcons'
import { formatBytes } from '@/lib/format'

import { Badge } from '@/components/ui/Badge'
import { IconButton } from '@/components/ui/Button'

import type { FolderInfo, TreeRow } from '@/features/torrent/tree'

export function Checkbox({
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
        'grid size-[18px] shrink-0 place-items-center rounded-[1px] border transition-colors',
        checked || indeterminate
          ? 'border-[var(--accent)] bg-[var(--accent)] text-[var(--accent-fg)]'
          : 'border-[var(--border-strong)] bg-[color-mix(in_srgb,var(--surface)_60%,transparent)] hover:border-[var(--accent)]',
      )}
    >
      {indeterminate ? (
        <Minus className="size-3" strokeWidth={3} />
      ) : checked ? (
        <Check className="size-3" strokeWidth={3} />
      ) : null}
    </button>
  )
}

function FileBadges({ file }: { file: FileSnapshot }) {
  return (
    <span className="flex shrink-0 items-center gap-1">
      {file.downloading && (
        <Badge tone="download" dot>
          downloading
        </Badge>
      )}
      {file.watched && <Badge tone="stream">watched</Badge>}
      {file.downloaded && <Badge tone="success">saved</Badge>}
    </span>
  )
}

export function TreeRowView({
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
        className="group flex h-[42px] items-center gap-2 rounded-[3px] px-2 hover:bg-[var(--hover)]"
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
        <span className="min-w-0 flex-1 truncate text-[13px] font-medium text-[var(--fg)]">
          {row.name}/
        </span>
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
      className="group flex h-[42px] items-center gap-2 rounded-[3px] px-2 hover:bg-[var(--hover)]"
      style={{ paddingLeft: indent + 30 }}
      onClick={() => onToggleFile(file)}
      role="treeitem"
      tabIndex={0}
    >
      <Checkbox
        checked={file.selected}
        onChange={() => onToggleFile(file)}
        label={`Select ${file.name}`}
      />
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
