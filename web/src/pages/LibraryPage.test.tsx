import { MemoryRouter, Route, Routes, useLocation } from 'react-router-dom'

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'

import type { LibraryEntry } from '@/types/api'

import { api } from '@/lib/api'

import { LibraryPage } from './LibraryPage'

import { mediaForFiles } from '@/test/fixtures'

vi.mock('@/lib/api', () => ({
  api: {
    library: vi.fn(),
    rescanLibrary: vi.fn(),
  },
}))

const entries: LibraryEntry[] = [
  {
    id: 'cache-6d6f766965',
    name: 'Movie.mkv',
    relativePath: 'Movies/Movie.mkv',
    root: 'cache',
    size: 2_147_483_648,
    modifiedMs: Date.now() - 60_000,
    streamPath: '/api/library/cache-6d6f766965/stream',
    media: mediaForFiles(
      { kind: 'library', value: 'cache-6d6f766965' },
      'Movie.mkv',
      'Movie.mkv',
      2_147_483_648,
      '/api/library/cache-6d6f766965/stream',
    ),
  },
  {
    id: 'downloads-73686f77',
    name: 'Show.mp4',
    relativePath: 'Show/Show.mp4',
    root: 'downloads',
    size: 1024,
    modifiedMs: null,
    streamPath: '/api/library/downloads-73686f77/stream',
    media: mediaForFiles(
      { kind: 'library', value: 'downloads-73686f77' },
      'Show.mp4',
      'Show.mp4',
      1024,
      '/api/library/downloads-73686f77/stream',
    ),
  },
]

function PlayerRoute() {
  const location = useLocation()
  return <div>Player route {location.search}</div>
}

function renderPage() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  return render(
    <QueryClientProvider client={client}>
      <MemoryRouter initialEntries={['/library']}>
        <Routes>
          <Route path="/library" element={<LibraryPage />} />
          <Route path="/player" element={<PlayerRoute />} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
})

describe('LibraryPage', () => {
  it('lists local files and filters by title', async () => {
    vi.mocked(api.library).mockResolvedValue(entries)
    renderPage()

    expect(await screen.findByText('Movie.mkv')).toBeInTheDocument()
    expect(screen.getByText('Show.mp4')).toBeInTheDocument()

    fireEvent.change(screen.getByLabelText('Filter local library'), {
      target: { value: 'show' },
    })
    await waitFor(() => expect(screen.queryByText('Movie.mkv')).not.toBeInTheDocument())
    expect(screen.getByText('Show.mp4')).toBeInTheDocument()
  })

  it('opens the global player for a library entry', async () => {
    vi.mocked(api.library).mockResolvedValue(entries)
    renderPage()

    const buttons = await screen.findAllByRole('button', { name: 'Play' })
    fireEvent.click(buttons[0])
    expect(await screen.findByText(/Player route/)).toBeInTheDocument()
    expect(screen.getByText(/library=cache-6d6f766965/)).toBeInTheDocument()
  })

  it('rescans and updates the list', async () => {
    vi.mocked(api.library).mockResolvedValue([])
    vi.mocked(api.rescanLibrary).mockResolvedValue([entries[0]])
    renderPage()

    expect(await screen.findByText('No local media yet')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: /rescan/i }))
    expect(await screen.findByText('Movie.mkv')).toBeInTheDocument()
    expect(api.rescanLibrary).toHaveBeenCalled()
  })
})
