import type { LucideIcon } from 'lucide-react'
import {
  Archive,
  BookOpen,
  Captions,
  Clapperboard,
  Disc3,
  File,
  FileCode2,
  FileImage,
  FileSpreadsheet,
  FileText,
  FileType2,
  Gamepad2,
  Music2,
  Package,
  Presentation,
  Tv,
  type LucideProps,
} from 'lucide-react'
import { cn } from '@/lib/cn'

export type FileKind =
  | 'movie'
  | 'tv'
  | 'audio'
  | 'image'
  | 'subtitle'
  | 'archive'
  | 'disc'
  | 'pdf'
  | 'ebook'
  | 'document'
  | 'spreadsheet'
  | 'presentation'
  | 'code'
  | 'app'
  | 'game'
  | 'font'
  | 'other'

const EXTENSIONS: Record<string, FileKind> = {
  mp4: 'movie', m4v: 'movie', mkv: 'movie', avi: 'movie', mov: 'movie', webm: 'movie',
  wmv: 'movie', flv: 'movie', mpg: 'movie', mpeg: 'movie', m2ts: 'movie', ts: 'movie',
  vob: 'movie', ogv: 'movie', rmvb: 'movie',
  mp3: 'audio', flac: 'audio', wav: 'audio', aac: 'audio', m4a: 'audio', mka: 'audio',
  ogg: 'audio', opus: 'audio', wma: 'audio', aiff: 'audio', alac: 'audio', ape: 'audio',
  jpg: 'image', jpeg: 'image', png: 'image', gif: 'image', webp: 'image', bmp: 'image',
  svg: 'image', avif: 'image', heic: 'image', tiff: 'image', tif: 'image',
  srt: 'subtitle', ass: 'subtitle', ssa: 'subtitle', vtt: 'subtitle', sub: 'subtitle',
  sup: 'subtitle', mks: 'subtitle', idx: 'subtitle',
  zip: 'archive', rar: 'archive', '7z': 'archive', tar: 'archive', gz: 'archive',
  bz2: 'archive', xz: 'archive', zst: 'archive', tgz: 'archive',
  iso: 'disc', img: 'disc', bin: 'disc', cue: 'disc', nrg: 'disc',
  pdf: 'pdf', epub: 'ebook', mobi: 'ebook', azw: 'ebook', azw3: 'ebook', fb2: 'ebook',
  djvu: 'ebook', cbz: 'ebook', cbr: 'ebook',
  doc: 'document', docx: 'document', txt: 'document', md: 'document', rtf: 'document',
  odt: 'document', log: 'document',
  xls: 'spreadsheet', xlsx: 'spreadsheet', csv: 'spreadsheet', ods: 'spreadsheet',
  ppt: 'presentation', pptx: 'presentation', odp: 'presentation', key: 'presentation',
  rs: 'code', tsx: 'code', js: 'code', jsx: 'code', mjs: 'code', cjs: 'code', py: 'code',
  go: 'code', java: 'code', c: 'code', cpp: 'code', h: 'code', hpp: 'code', css: 'code',
  scss: 'code', html: 'code', json: 'code', yaml: 'code', yml: 'code', toml: 'code',
  sh: 'code', bash: 'code', zsh: 'code', lua: 'code', php: 'code', rb: 'code', sql: 'code',
  exe: 'app', msi: 'app', dmg: 'app', appimage: 'app', deb: 'app', rpm: 'app', apk: 'app',
  nes: 'game', sfc: 'game', gba: 'game', nds: 'game', '3ds': 'game', wad: 'game',
  n64: 'game', iso_game: 'game',
  ttf: 'font', otf: 'font', woff: 'font', woff2: 'font',
}

const ICONS: Record<FileKind, LucideIcon> = {
  movie: Clapperboard,
  tv: Tv,
  audio: Music2,
  image: FileImage,
  subtitle: Captions,
  archive: Archive,
  disc: Disc3,
  pdf: FileText,
  ebook: BookOpen,
  document: FileText,
  spreadsheet: FileSpreadsheet,
  presentation: Presentation,
  code: FileCode2,
  app: Package,
  game: Gamepad2,
  font: FileType2,
  other: File,
}

const TONES: Record<FileKind, string> = {
  movie: 'var(--accent)',
  tv: 'var(--badge-stream)',
  audio: 'var(--warning)',
  image: 'var(--folder)',
  subtitle: 'var(--muted-strong)',
  archive: 'var(--muted)',
  disc: 'var(--info)',
  pdf: 'var(--danger)',
  ebook: 'var(--success)',
  document: 'var(--muted-strong)',
  spreadsheet: 'var(--success)',
  presentation: 'var(--warning)',
  code: 'var(--success)',
  app: 'var(--accent)',
  game: 'var(--badge-stream)',
  font: 'var(--muted-strong)',
  other: 'var(--muted)',
}

export function extensionOf(name: string): string {
  const base = name.split('/').pop() ?? name
  const dot = base.lastIndexOf('.')
  return dot > 0 ? base.slice(dot + 1).toLowerCase() : ''
}

const TV_PATTERN = /\b(s\d{1,2}(?:\s*e\d{1,3})?(?:\s*-\s*s?\d{1,2})?|season\s*\d+|complete series|episode\s*\d+)\b/i

export function fileKind(name: string): FileKind {
  const kind = EXTENSIONS[extensionOf(name)]
  if (!kind) return 'other'
  if (kind === 'movie' && TV_PATTERN.test(name)) return 'tv'
  return kind
}

/** Guess the icon for a torrent title, which usually has no file extension. */
export function mediaKind(name: string): FileKind {
  const lower = name.toLowerCase()
  if (TV_PATTERN.test(lower)) return 'tv'
  if (
    /\b(1080p|2160p|720p|480p|bluray|blu-ray|web[- .]?dl|webrip|hdtv|bdrip|dvdrip|x265|hevc|h\.?264|remux|xvid|divx)\b/.test(
      lower,
    ) ||
    /\(\d{4}\)/.test(lower)
  ) {
    return 'movie'
  }
  if (/\b(flac|mp3|ost|soundtrack|album|discography|aac|vinyl)\b/.test(lower)) return 'audio'
  if (/\b(epub|mobi|azw3?|pdf|book|novel|bible|handbook|magazine|comic)\b/.test(lower)) return 'ebook'
  if (/\b(game|repack|fitgirl|codex|skidrow|empress|gog|switch|rom)\b/.test(lower)) return 'game'
  if (/\b(iso|disc image)\b/.test(lower)) return 'disc'
  return 'other'
}

export function fileIcon(name: string): LucideIcon {
  return ICONS[fileKind(name)]
}

export function mediaIcon(name: string): LucideIcon {
  const kind = mediaKind(name)
  return kind === 'other' ? Clapperboard : ICONS[kind]
}

export function iconTone(kind: FileKind): string {
  return TONES[kind]
}

export function mediaTone(name: string): string {
  return TONES[mediaKind(name)]
}

export function isVideo(kind: FileKind): boolean {
  return kind === 'movie' || kind === 'tv'
}

export function isAudio(kind: FileKind): boolean {
  return kind === 'audio'
}

export function isImage(kind: FileKind): boolean {
  return kind === 'image'
}

export function canPreview(kind: FileKind): boolean {
  return isVideo(kind) || isAudio(kind) || isImage(kind)
}

export function previewLabel(kind: FileKind): string {
  if (isAudio(kind)) return 'Play audio'
  if (isImage(kind)) return 'Preview image'
  return 'Watch video'
}

export function kindLabel(kind: FileKind): string {
  switch (kind) {
    case 'movie': return 'Movie'
    case 'tv': return 'TV'
    case 'audio': return 'Audio'
    case 'image': return 'Image'
    case 'subtitle': return 'Subtitle'
    case 'archive': return 'Archive'
    case 'disc': return 'Disc'
    case 'pdf': return 'PDF'
    case 'ebook': return 'Book'
    case 'document': return 'Document'
    case 'spreadsheet': return 'Spreadsheet'
    case 'presentation': return 'Slides'
    case 'code': return 'Code'
    case 'app': return 'App'
    case 'game': return 'Game'
    case 'font': return 'Font'
    default: return 'File'
  }
}

interface FileIconProps extends Omit<LucideProps, 'name'> {
  name: string
  className?: string
}

/** Icon for a real file path, inferred from its extension. */
export function FileTypeIcon({ name, className, ...props }: FileIconProps) {
  const kind = fileKind(name)
  const Icon = ICONS[kind]
  return (
    <Icon
      {...props}
      className={cn('shrink-0', className)}
      style={{ color: TONES[kind], ...props.style }}
      aria-hidden
    />
  )
}

/** Icon for a torrent title, which usually has no extension. */
export function MediaIcon({ name, className, ...props }: FileIconProps) {
  const kind = mediaKind(name)
  const Icon = kind === 'other' ? Clapperboard : ICONS[kind]
  return (
    <Icon
      {...props}
      className={cn('shrink-0', className)}
      style={{ color: TONES[kind], ...props.style }}
      aria-hidden
    />
  )
}

