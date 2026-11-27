import type { DownloadSnapshot, MediaDescriptor, MediaKind, PlaybackMode } from '@/types/api'

const VIDEO = [
  'mp4',
  'm4v',
  'mkv',
  'webm',
  'avi',
  'mov',
  'wmv',
  'flv',
  'mpg',
  'mpeg',
  'm2ts',
  'ts',
  'vob',
  'ogv',
  'rmvb',
]
const AUDIO = [
  'mp3',
  'flac',
  'wav',
  'aac',
  'm4a',
  'mka',
  'ogg',
  'oga',
  'opus',
  'wma',
  'aiff',
  'alac',
  'ape',
]
const IMAGE = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg', 'avif', 'heic', 'tiff', 'tif']
const DIRECT_VIDEO = ['mp4', 'm4v', 'webm', 'ogv']

function extension(name: string): string {
  return name.split('.').pop()?.toLowerCase() ?? ''
}

export function mediaForFiles(
  id: MediaDescriptor['id'],
  title: string,
  filename: string,
  size: number,
  streamPath: string | null,
): MediaDescriptor {
  const ext = extension(filename)
  const kind: MediaKind = VIDEO.includes(ext)
    ? 'video'
    : AUDIO.includes(ext)
      ? 'audio'
      : IMAGE.includes(ext)
        ? 'image'
        : 'other'
  const playback: PlaybackMode = kind === 'video' && !DIRECT_VIDEO.includes(ext) ? 'hls' : 'direct'
  const resolvedStreamPath =
    streamPath && playback === 'hls'
      ? id.kind === 'download'
        ? `/api/transcode/download/${id.value}/index.m3u8`
        : `/api/transcode/library/${encodeURIComponent(id.value)}/index.m3u8`
      : streamPath
  return { id, title, filename, kind, playback, streamPath: resolvedStreamPath, size }
}

export function withMedia(download: Omit<DownloadSnapshot, 'media'>): DownloadSnapshot {
  const filename = download.fileNames[0] ?? download.label
  return {
    ...download,
    media: mediaForFiles(
      { kind: 'download', value: download.id },
      download.label,
      filename,
      download.totalSize,
      download.streamPath,
    ),
  }
}
