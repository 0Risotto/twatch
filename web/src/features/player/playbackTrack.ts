import type {
  DownloadSnapshot,
  LibraryEntry,
  MediaDescriptor,
  QueueFile,
  SubtitleTrack,
} from '@/types/api'

export interface PlaybackTrack {
  source: 'download' | 'library'
  key: string
  media: MediaDescriptor
  title: string
  subtitle: string
  historyUrl: string
  streamPath: string | null
  directStreamPath: string | null
  size: number
  resumePosition: number | null
  torrentId: number | null
  torrentName: string
  subtitles: SubtitleTrack[]
  queue: QueueFile[]
  fileIndex: number
  download: DownloadSnapshot | null
  library: LibraryEntry | null
}

export function downloadTrack(download: DownloadSnapshot): PlaybackTrack {
  return {
    source: 'download',
    key: `download:${download.id}`,
    media: download.media,
    title: download.label,
    subtitle: download.torrentName,
    historyUrl: download.url,
    streamPath: download.media.streamPath,
    directStreamPath: download.streamPath,
    size: download.totalSize,
    resumePosition: download.resumePosition,
    torrentId: download.torrentId,
    torrentName: download.torrentName,
    subtitles: download.subtitles,
    queue: download.queue,
    fileIndex: download.fileIndex,
    download,
    library: null,
  }
}

export function libraryTrack(entry: LibraryEntry): PlaybackTrack {
  return {
    source: 'library',
    key: `library:${entry.id}`,
    media: entry.media,
    title: entry.name,
    subtitle: 'Local library',
    historyUrl: '',
    streamPath: entry.media.streamPath,
    directStreamPath: entry.streamPath,
    size: entry.size,
    resumePosition: null,
    torrentId: null,
    torrentName: entry.relativePath,
    subtitles: [],
    queue: [],
    fileIndex: 0,
    download: null,
    library: entry,
  }
}
