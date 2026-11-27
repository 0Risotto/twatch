import { describe, expect, it, vi } from 'vitest'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { DownloadSnapshot } from '@/types/api'

const mockState = vi.hoisted(() => {
  const instances: Array<{
    options: Record<string, unknown>
    timeOffset: number
    destroyed: boolean
    resize: ReturnType<typeof vi.fn>
    destroy: ReturnType<typeof vi.fn>
    setCurrentTime: ReturnType<typeof vi.fn>
  }> = []

  class MockJassub extends EventTarget {
    options: Record<string, unknown>
    timeOffset = 0
    destroyed = false
    resize = vi.fn()
    destroy = vi.fn(() => {
      this.destroyed = true
    })
    setCurrentTime = vi.fn()

    constructor(options: Record<string, unknown>) {
      super()
      this.options = options
      instances.push(this)
    }
  }

  return { instances, MockJassub }
})

vi.mock('jassub', () => ({ default: mockState.MockJassub }))
vi.mock('jassub/dist/jassub-worker.js?url', () => ({ default: '/assets/jassub-worker.js' }))
vi.mock('jassub/dist/jassub-worker.wasm?url', () => ({ default: '/assets/jassub-worker.wasm' }))
vi.mock('jassub/dist/jassub-worker.wasm.js?url', () => ({
  default: '/assets/jassub-worker.wasm.js',
}))
vi.mock('jassub/dist/jassub-worker-modern.wasm?url', () => ({
  default: '/assets/jassub-worker-modern.wasm',
}))
vi.mock('jassub/dist/default.woff2?url', () => ({ default: '/assets/default.woff2' }))

import { withMedia } from '@/test/fixtures'
import { useSubtitleTracks } from './useSubtitleTracks'

const download: DownloadSnapshot = withMedia({
  id: 1,
  torrentId: 7,
  torrentName: 'Test Torrent',
  label: 'episode.ass',
  fileNames: ['episode.ass'],
  url: 'magnet:test',
  progress: 1,
  speed: 0,
  uploadSpeed: 0,
  peers: 0,
  totalSize: 100,
  downloaded: 100,
  etaSeconds: null,
  isStreaming: true,
  state: 'streaming',
  streamPath: '/api/stream/7/0',
  subtitles: [{ index: 2, name: 'episode.en.ass' }],
  error: null,
  fileIndex: 0,
  queue: [],
  resumePosition: null,
})

function refs() {
  const media = document.createElement('video')
  Object.defineProperty(media, 'textTracks', { value: [] })
  return {
    media,
    mediaRef: { current: media as HTMLMediaElement | null },
    containerRef: { current: document.createElement('div') },
    canvasRef: { current: document.createElement('canvas') },
  }
}

describe('useSubtitleTracks ASS rendering', () => {
  it('creates JASSUB for an ASS track and forwards delay changes', async () => {
    mockState.instances.length = 0
    const { media, mediaRef, containerRef, canvasRef } = refs()
    const { result } = renderHook(() =>
      useSubtitleTracks(download, mediaRef, containerRef, canvasRef, null),
    )

    await waitFor(() => expect(mockState.instances).toHaveLength(1))
    const instance = mockState.instances[0]
    expect(instance.options.subUrl).toBe('/api/subtitles/7/2?format=raw')
    expect(instance.options.video).toBe(media)
    expect(instance.options.canvas).toBe(canvasRef.current)

    act(() => result.current.adjustDelay(1.5))
    await waitFor(() => expect(instance.timeOffset).toBe(1.5))
    expect(mockState.instances).toHaveLength(1)
  })

  it('destroys JASSUB when subtitles are turned off', async () => {
    mockState.instances.length = 0
    const { mediaRef, containerRef, canvasRef } = refs()
    const { result } = renderHook(() =>
      useSubtitleTracks(download, mediaRef, containerRef, canvasRef, null),
    )

    await waitFor(() => expect(mockState.instances).toHaveLength(1))
    const instance = mockState.instances[0]
    act(() => result.current.selectTrack(null))
    await waitFor(() => expect(instance.destroy).toHaveBeenCalled())
  })

  it('passes external ASS content to JASSUB and exposes a native fallback track', async () => {
    mockState.instances.length = 0
    const { mediaRef, containerRef, canvasRef } = refs()
    const external = {
      name: 'external.en.ass',
      content:
        '[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:03.00,Default,,0,0,0,,External line\n',
    }
    const { result } = renderHook(() =>
      useSubtitleTracks(
        { ...download, subtitles: [] },
        mediaRef,
        containerRef,
        canvasRef,
        external,
      ),
    )

    expect(result.current.tracks).toHaveLength(1)
    await waitFor(() => expect(result.current.selected).toBe('external.en.ass'))
    await waitFor(() => expect(mockState.instances).toHaveLength(1))
    const instance = mockState.instances[0]
    expect(instance.options.subContent).toBe(external.content)
    expect(instance.options.subUrl).toBeUndefined()
    expect(result.current.nativeTracks).toHaveLength(1)
    expect(result.current.nativeTracks[0].url).toContain('data:text/vtt')
    expect(result.current.nativeTracks[0].label).toContain('external')
  })
})
