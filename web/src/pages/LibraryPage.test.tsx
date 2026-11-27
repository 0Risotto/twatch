import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { LibraryPage } from './LibraryPage'
import { api } from '@/lib/api'
import type { LibraryEntry } from '@/types/api'

vi.mock('@/lib/api', () => ({
  api: {
    library: vi.fn(),
    rescanLibrary: vi.fn(),
    openLibrary: vi.fn(),
  },
}))

const entries: LibraryEntry[] = [
  {
    id: 'cache-6d6f766965',
    name: 'Movie.mkv',
    relativePath: 'Movies/Movie.mkv',
    root: 'cache',
    mediaKind: 'video',
    size: 2_147_483_648,
    modifiedMs: Date.now() - 60_000,
    streamPath: '/api/library/cache-6d6f766965/stream',
  },
  {
    id: 'downloads-73686f77',
    name: 'Show.mp4',
    relativePath: 'Show/Show.mp4',
    root: 'downloads',
    mediaKind: 'video',
    size: 1024,
    modifiedMs: null,
    streamPath: '/api/library/downloads-73686f77/stream',
  },
]

function renderPage() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  return render(
    <QueryClientProvider client={client}>
      <LibraryPage />
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

  it('opens unsupported formats in the native player', async () => {
    vi.mocked(api.library).mockResolvedValue(entries)
    vi.mocked(api.openLibrary).mockResolvedValue({
      running: false,
      title: 'Movie.mkv',
      streamPath: '/api/library/cache-6d6f766965/stream',
      downloadId: null,
    })
    renderPage()

    const buttons = await screen.findAllByRole('button', { name: /mpv \/ vlc/i })
    fireEvent.click(buttons[0])
    await waitFor(() => expect(api.openLibrary).toHaveBeenCalledWith('cache-6d6f766965'))
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
