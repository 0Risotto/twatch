import type { FileSnapshot } from '@/types/api'

export interface FolderInfo {
  path: string
  fileIndices: number[]
  size: number
  selected: number
}

export interface TreeRow {
  key: string
  type: 'folder' | 'file'
  name: string
  depth: number
  file?: FileSnapshot
  folder?: FolderInfo
}

interface Node {
  name: string
  path: string
  folders: Map<string, Node>
  files: FileSnapshot[]
}

function emptyNode(name: string, path: string): Node {
  return { name, path, folders: new Map(), files: [] }
}

function insert(root: Node, file: FileSnapshot) {
  const parts = file.name.split('/').filter(Boolean)
  let node = root
  for (const part of parts.slice(0, -1)) {
    const path = node.path ? `${node.path}/${part}` : part
    if (!node.folders.has(part)) node.folders.set(part, emptyNode(part, path))
    node = node.folders.get(part)!
  }
  node.files.push(file)
}

function hasMatch(node: Node, needle: string): boolean {
  if (node.files.some(file => file.name.toLowerCase().includes(needle))) return true
  for (const folder of node.folders.values()) {
    if (hasMatch(folder, needle)) return true
  }
  return false
}

function folderInfo(node: Node): FolderInfo {
  const fileIndices: number[] = []
  let size = 0
  let selected = 0
  const visit = (current: Node) => {
    for (const file of current.files) {
      fileIndices.push(file.index)
      size += file.size
      if (file.selected) selected += 1
    }
    for (const child of current.folders.values()) visit(child)
  }
  visit(node)
  return { path: node.path, fileIndices, size, selected }
}

function walk(
  node: Node,
  depth: number,
  expanded: Set<string>,
  needle: string,
  rows: TreeRow[],
) {
  const folders = [...node.folders.values()].sort((a, b) => a.name.localeCompare(b.name))
  for (const folder of folders) {
    if (needle && !hasMatch(folder, needle)) continue
    rows.push({
      key: `folder:${folder.path}`,
      type: 'folder',
      name: folder.name,
      depth,
      folder: folderInfo(folder),
    })
    if (needle || expanded.has(folder.path)) {
      walk(folder, depth + 1, expanded, needle, rows)
    }
  }

  const files = node.files
    .filter(file => !needle || file.name.toLowerCase().includes(needle))
    .sort((a, b) => a.name.localeCompare(b.name))
  for (const file of files) {
    rows.push({
      key: `file:${file.index}`,
      type: 'file',
      name: file.name.split('/').pop() ?? file.name,
      depth,
      file,
    })
  }
}

export function buildTreeRows(
  files: FileSnapshot[],
  expanded: Set<string>,
  filter: string,
): TreeRow[] {
  const root = emptyNode('', '')
  for (const file of files) insert(root, file)
  const rows: TreeRow[] = []
  walk(root, 0, expanded, filter.trim().toLowerCase(), rows)
  return rows
}

export function allFolderPaths(files: FileSnapshot[]): string[] {
  const paths = new Set<string>()
  for (const file of files) {
    const parts = file.name.split('/').filter(Boolean)
    for (let index = 1; index < parts.length; index += 1) {
      paths.add(parts.slice(0, index).join('/'))
    }
  }
  return [...paths]
}
