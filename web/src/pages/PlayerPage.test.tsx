import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, render } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { PlayerPage } from './PlayerPage'
import { PlaybackProvider } from '@/features/player/PlaybackProvider'
import { useAppStore } from '@/store/app'
import { withMedia } from '@/test/fixtures'
import type { DownloadSnapshot } from '@/types/api'

const base: Omit<DownloadSnapshot, 'media'> = {
  id: 1,
  torrentId: 7,
  torrentName: 'Test Torrent',
  label: 'file',
  fileNames: ['file'],
  url: 'magnet:test',
  progress: 0.5,
  speed: 1024,
  uploadSpeed: 0,
  peers: 2,
  totalSize: 2048,
  downloaded: 1024,
  etaSeconds: 1,
  isStreaming: true,
  state: 'streaming',
  streamPath: '/api/stream/7/0',
  subtitles: [],
  error: null,
  fileIndex: 0,
  queue: [],
  resumePosition: null,
}

function renderWith(download: Omit<DownloadSnapshot, 'media'>) {
  const item = withMedia(download)
  useAppStore.setState({
    downloads: [item],
  })
  return render(
    <MemoryRouter initialEntries={[`/player?download=${item.id}`]}>
      <PlaybackProvider>
        <PlayerPage />
      </PlaybackProvider>
    </MemoryRouter>,
  )
}

afterEach(cleanup)

describe('PlayerPage media modes', () => {
  it('renders video for a movie file', () => {
    const { container } = renderWith({ ...base, label: 'movie.mp4', fileNames: ['movie.mp4'] })
    expect(container.querySelector('video')).not.toBeNull()
  })

  it('routes an MKV through the local transcode player', () => {
    const { container } = renderWith({ ...base, label: 'movie.mkv', fileNames: ['movie.mkv'] })
    expect(container.querySelector('video')).not.toBeNull()
    expect(container.innerHTML).not.toContain('This video needs a native player')
  })

  it('renders an audio element for an audio file', () => {
    const { container } = renderWith({ ...base, label: 'track.flac', fileNames: ['track.flac'] })
    expect(container.querySelector('audio')).not.toBeNull()
  })

  it('attaches in-torrent subtitles to the video element', () => {
    const { container } = renderWith({
      ...base,
      label: 'episode.mp4',
      fileNames: ['episode.mp4'],
      subtitles: [
        { index: 2, name: 'episode.en.srt' },
        { index: 3, name: 'episode.es.srt' },
      ],
    })
    const tracks = [...container.querySelectorAll('track')]
    expect(tracks).toHaveLength(2)
    expect(tracks[0].getAttribute('src')).toBe('/api/subtitles/7/2')
    expect(tracks[0].getAttribute('srclang')).toBe('en')
    expect(tracks[0].getAttribute('label')).toBe('English')
  })

  it('renders an image for an image file', () => {
    const { container } = renderWith({ ...base, label: 'cover.jpg', fileNames: ['cover.jpg'] })
    const image = container.querySelector('img')
    expect(image).not.toBeNull()
    expect(image?.getAttribute('src')).toBe('/api/stream/7/0')
  })

  it('shows the open-file fallback for a PDF', () => {
    const { container } = renderWith({ ...base, label: 'book.pdf', fileNames: ['book.pdf'] })
    expect(container.querySelector('video')).toBeNull()
    expect(container.querySelector('audio')).toBeNull()
    expect(container.innerHTML).toContain('PDF preview is not available')
  })
})
