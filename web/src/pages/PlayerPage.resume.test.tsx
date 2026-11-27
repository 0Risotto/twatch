import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { PlayerPage } from './PlayerPage'
import { PlaybackProvider } from '@/features/player/PlaybackProvider'
import { useAppStore } from '@/store/app'
import type { DownloadSnapshot } from '@/types/api'

const paused: DownloadSnapshot = {
  id: 9,
  torrentId: 7,
  torrentName: 'Test Torrent',
  label: 'restored.mp4',
  fileNames: ['restored.mp4'],
  url: 'magnet:test',
  progress: 0.56,
  speed: 0,
  uploadSpeed: 0,
  peers: 0,
  totalSize: 2048,
  downloaded: 1146,
  etaSeconds: null,
  isStreaming: true,
  state: 'paused',
  streamPath: null,
  subtitles: [],
  error: null,
  fileIndex: 0,
  queue: [],
  resumePosition: null,
}

const streaming: DownloadSnapshot = {
  ...paused,
  state: 'streaming',
  streamPath: '/api/stream/7/0',
}

function response(body: unknown): Response {
  return { ok: true, status: 200, json: async () => body } as Response
}

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
})

describe('PlaybackProvider stream recovery', () => {
  it('resumes a restored paused download and attaches its stream', async () => {
    let downloadRequests = 0
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input)
      if (url === '/api/downloads') {
        downloadRequests += 1
        return response(downloadRequests === 1 ? [paused] : [streaming])
      }
      if (url === '/api/downloads/9/resume') return response({ ok: true })
      throw new Error(`Unexpected request: ${url}`)
    })
    vi.stubGlobal('fetch', fetchMock)

    useAppStore.setState({
      downloads: [paused],
      player: { running: false, title: null, streamPath: null, downloadId: null },
    })

    const { container } = render(
      <MemoryRouter initialEntries={['/player?download=9']}>
        <PlaybackProvider>
          <PlayerPage />
        </PlaybackProvider>
      </MemoryRouter>,
    )

    expect(container.textContent).toContain('Preparing movie')
    await waitFor(() => expect(container.querySelector('video')).not.toBeNull(), {
      timeout: 5_000,
    })
    expect(fetchMock).toHaveBeenCalledWith('/api/downloads/9/resume', expect.anything())
  })
})
