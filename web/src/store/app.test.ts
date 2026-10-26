import { beforeEach, describe, expect, it } from 'vitest'
import { useAppStore } from './app'
import type { DownloadSnapshot, FileSnapshot, SessionSnapshot } from '@/types/api'

function file(index: number, name: string): FileSnapshot {
  return {
    index,
    name,
    size: 100,
    selected: false,
    watched: false,
    downloaded: false,
    downloading: false,
  }
}

function session(): SessionSnapshot {
  return {
    url: 'magnet:test',
    name: 'Test',
    files: [file(0, 'one.mkv'), file(1, 'two.mkv')],
    selectedCount: 0,
    selectedSize: 0,
    totalSize: 200,
    downloadRoot: 'Test',
  }
}

const download: DownloadSnapshot = {
  id: 1,
  torrentId: 1,
  torrentName: 'Test',
  label: 'one.mkv',
  fileNames: ['one.mkv'],
  url: 'magnet:test',
  progress: 0.25,
  speed: 10,
  uploadSpeed: 0,
  peers: 1,
  totalSize: 100,
  downloaded: 25,
  etaSeconds: 8,
  isStreaming: true,
  state: 'streaming',
  streamPath: '/api/stream/1/0',
  subtitles: [],
  error: null,
}

beforeEach(() => {
  useAppStore.setState({
    booted: false,
    bootstrap: null,
    session: null,
    downloads: [],
    player: { running: false, title: null, streamPath: null, downloadId: null },
    config: null,
    themes: [],
    searchConfigured: false,
    connected: false,
    revision: 0,
    toasts: [],
  })
})

describe('realtime store', () => {
  it('applies a snapshot and ignores stale frames', () => {
    useAppStore.getState().applyFrame({
      type: 'snapshot',
      revision: 5,
      session: session(),
      downloads: [download],
      player: { running: false, title: null, streamPath: null, downloadId: null },
    })
    expect(useAppStore.getState().revision).toBe(5)
    expect(useAppStore.getState().downloads).toHaveLength(1)

    useAppStore.getState().applyFrame({
      type: 'session',
      revision: 4,
      session: null,
    })
    expect(useAppStore.getState().session).not.toBeNull()

    useAppStore.getState().applyFrame({
      type: 'downloads',
      revision: 6,
      downloads: [],
    })
    expect(useAppStore.getState().downloads).toHaveLength(0)
  })

  it('updates selection optimistically', () => {
    useAppStore.setState({ session: session() })
    useAppStore.getState().optimisticSelect([0], true)
    const next = useAppStore.getState().session!
    expect(next.selectedCount).toBe(1)
    expect(next.selectedSize).toBe(100)
    expect(next.files[0].selected).toBe(true)
  })

  it('adds and dismisses toasts', () => {
    const id = useAppStore.getState().addToast('success', 'Done')
    expect(useAppStore.getState().toasts).toHaveLength(1)
    useAppStore.getState().dismissToast(id)
    expect(useAppStore.getState().toasts).toHaveLength(0)
  })
})
