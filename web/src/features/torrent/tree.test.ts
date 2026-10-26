import { describe, expect, it } from 'vitest'
import { allFolderPaths, buildTreeRows } from './tree'
import type { FileSnapshot } from '@/types/api'

function file(index: number, name: string, selected = false): FileSnapshot {
  return {
    index,
    name,
    size: 100,
    selected,
    watched: false,
    downloaded: false,
    downloading: false,
  }
}

describe('torrent tree', () => {
  const files = [
    file(0, 'Movie.mkv'),
    file(1, 'Extras/Featurette.mkv'),
    file(2, 'Extras/Behind The Scenes.mkv', true),
  ]

  it('keeps collapsed folders to one row', () => {
    const rows = buildTreeRows(files, new Set(), '')
    expect(rows.map(row => row.name)).toEqual(['Extras', 'Movie.mkv'])
    expect(rows[0].folder?.fileIndices).toEqual([1, 2])
    expect(rows[0].folder?.selected).toBe(1)
  })

  it('expands folders and filters files', () => {
    const rows = buildTreeRows(files, new Set(['Extras']), '')
    expect(rows.map(row => row.name)).toEqual([
      'Extras',
      'Behind The Scenes.mkv',
      'Featurette.mkv',
      'Movie.mkv',
    ])

    const filtered = buildTreeRows(files, new Set(), 'behind')
    expect(filtered.map(row => row.name)).toEqual(['Extras', 'Behind The Scenes.mkv'])
  })

  it('lists every folder path', () => {
    expect(allFolderPaths(files)).toEqual(['Extras'])
  })
})
