import { MemoryRouter } from 'react-router-dom'

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { cleanup, render, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'

import type { LibraryEntry } from '@/types/api'

import { api } from '@/lib/api'

import { PlaybackProvider } from '@/features/player/PlaybackProvider'

import { PlayerPage } from './PlayerPage'

import { mediaForFiles } from '@/test/fixtures'

const entry: LibraryEntry = {
  id: 'cache-movie',
  name: 'Movie.mkv',
  relativePath: 'Movies/Movie.mkv',
  root: 'cache',
  size: 2_147_483_648,
  modifiedMs: Date.now() - 60_000,
  streamPath: '/api/library/cache-movie/stream',
  media: mediaForFiles(
    { kind: 'library', value: 'cache-movie' },
    'Movie.mkv',
    'Movie.mkv',
    2_147_483_648,
    '/api/library/cache-movie/stream',
  ),
}

afterEach(() => {
  cleanup()
  vi.restoreAllMocks()
})

describe('PlayerPage library playback', () => {
  it('loads a library entry through the global player', async () => {
    const library = vi.spyOn(api, 'library').mockResolvedValue([entry])
    const { container } = render(
      <MemoryRouter initialEntries={['/player?library=cache-movie']}>
        <QueryClientProvider client={new QueryClient()}>
          <PlaybackProvider>
            <PlayerPage />
          </PlaybackProvider>
        </QueryClientProvider>
      </MemoryRouter>,
    )

    await waitFor(() => expect(container.querySelector('video')).not.toBeNull())
    expect(library).toHaveBeenCalled()
    expect(container.textContent).toContain('Movie.mkv')
    expect(container.textContent).toContain('Local library')
    expect(container.textContent).not.toContain('Stream health')
  })
})
