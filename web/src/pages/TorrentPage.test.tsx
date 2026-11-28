import { MemoryRouter } from 'react-router-dom'

import { cleanup, render, screen } from '@testing-library/react'
import { afterEach, describe, expect, it } from 'vitest'

import type { FileSnapshot, SessionSnapshot } from '@/types/api'

import { useAppStore } from '@/store/app'

import { TorrentPage } from './TorrentPage'

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

function renderSelected(name: string, index = 0) {
  const session: SessionSnapshot = {
    url: 'magnet:test',
    name: 'Test',
    files: [file(index, name, true)],
    selectedCount: 1,
    selectedSize: 100,
    totalSize: 100,
    downloadRoot: null,
  }
  useAppStore.setState({
    session,
    downloads: [],
  })
  return render(
    <MemoryRouter>
      <TorrentPage />
    </MemoryRouter>,
  )
}

afterEach(cleanup)

describe('TorrentPage open action', () => {
  it('offers Watch video for a movie', () => {
    renderSelected('movie.mkv')
    expect(screen.getAllByRole('button', { name: /Watch video/i })[0]).toBeEnabled()
  })

  it('offers Play audio for audio', () => {
    renderSelected('track.flac')
    expect(screen.getAllByRole('button', { name: /Play audio/i })[0]).toBeEnabled()
  })

  it('offers Preview image for images', () => {
    renderSelected('cover.jpg')
    expect(screen.getAllByRole('button', { name: /Preview image/i })[0]).toBeEnabled()
  })

  it('does not offer preview for a PDF', () => {
    renderSelected('book.pdf')
    expect(screen.queryAllByRole('button', { name: /^Watch$/i })).toHaveLength(0)
  })
})
