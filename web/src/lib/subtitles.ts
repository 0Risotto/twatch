import type { SubtitleTrack } from '@/types/api'

const SUBTITLE_EXTENSIONS = new Set(['srt', 'ass', 'ssa', 'vtt'])

export function isSubtitleFile(name: string): boolean {
  const extension = name.split('/').pop()?.split('.').pop()?.toLowerCase() ?? ''
  return SUBTITLE_EXTENSIONS.has(extension)
}

export function stripExtension(name: string): string {
  const base = name.split('/').pop() ?? name
  const dot = base.lastIndexOf('.')
  return dot > 0 ? base.slice(0, dot) : base
}

const LANGUAGE_NAMES: Record<string, string> = {
  en: 'English',
  eng: 'English',
  english: 'English',
  es: 'Spanish',
  spa: 'Spanish',
  esp: 'Spanish',
  fr: 'French',
  fre: 'French',
  fra: 'French',
  de: 'German',
  ger: 'German',
  deu: 'German',
  it: 'Italian',
  ita: 'Italian',
  pt: 'Portuguese',
  por: 'Portuguese',
  ptbr: 'Portuguese (BR)',
  pb: 'Portuguese (BR)',
  nl: 'Dutch',
  dut: 'Dutch',
  nld: 'Dutch',
  pl: 'Polish',
  pol: 'Polish',
  ru: 'Russian',
  rus: 'Russian',
  uk: 'Ukrainian',
  ukr: 'Ukrainian',
  ja: 'Japanese',
  jpn: 'Japanese',
  jp: 'Japanese',
  ko: 'Korean',
  kor: 'Korean',
  zh: 'Chinese',
  chi: 'Chinese',
  zho: 'Chinese',
  chs: 'Chinese (Simplified)',
  cht: 'Chinese (Traditional)',
  ar: 'Arabic',
  ara: 'Arabic',
  he: 'Hebrew',
  heb: 'Hebrew',
  hi: 'Hindi',
  hin: 'Hindi',
  ta: 'Tamil',
  tam: 'Tamil',
  te: 'Telugu',
  tel: 'Telugu',
  th: 'Thai',
  tha: 'Thai',
  vi: 'Vietnamese',
  vie: 'Vietnamese',
  id: 'Indonesian',
  ind: 'Indonesian',
  ms: 'Malay',
  may: 'Malay',
  tr: 'Turkish',
  tur: 'Turkish',
  sv: 'Swedish',
  swe: 'Swedish',
  da: 'Danish',
  dan: 'Danish',
  fi: 'Finnish',
  fin: 'Finnish',
  no: 'Norwegian',
  nor: 'Norwegian',
  cs: 'Czech',
  ces: 'Czech',
  cze: 'Czech',
  hu: 'Hungarian',
  hun: 'Hungarian',
  ro: 'Romanian',
  ron: 'Romanian',
  rum: 'Romanian',
  el: 'Greek',
  gre: 'Greek',
  ell: 'Greek',
}

const ISO_CODES: Record<string, string> = {
  english: 'en',
  eng: 'en',
  spanish: 'es',
  spa: 'es',
  esp: 'es',
  french: 'fr',
  fre: 'fr',
  fra: 'fr',
  german: 'de',
  ger: 'de',
  deu: 'de',
  italian: 'it',
  ita: 'it',
  portuguese: 'pt',
  por: 'pt',
  ptbr: 'pt-BR',
  dutch: 'nl',
  dut: 'nl',
  nld: 'nl',
  polish: 'pl',
  pol: 'pl',
  russian: 'ru',
  rus: 'ru',
  ukrainian: 'uk',
  ukr: 'uk',
  japanese: 'ja',
  jpn: 'ja',
  jp: 'ja',
  korean: 'ko',
  kor: 'ko',
  chinese: 'zh',
  chi: 'zh',
  zho: 'zh',
  chs: 'zh-CN',
  cht: 'zh-TW',
  arabic: 'ar',
  ara: 'ar',
  hebrew: 'he',
  heb: 'he',
  hindi: 'hi',
  hin: 'hi',
  tamil: 'ta',
  tam: 'ta',
  telugu: 'te',
  tel: 'te',
  thai: 'th',
  tha: 'th',
  vietnamese: 'vi',
  vie: 'vi',
  indonesian: 'id',
  ind: 'id',
  malay: 'ms',
  may: 'ms',
  turkish: 'tr',
  tur: 'tr',
  swedish: 'sv',
  swe: 'sv',
  danish: 'da',
  dan: 'da',
  finnish: 'fi',
  fin: 'fi',
  norwegian: 'no',
  nor: 'no',
  czech: 'cs',
  ces: 'cs',
  cze: 'cs',
  hungarian: 'hu',
  hun: 'hu',
  romanian: 'ro',
  ron: 'ro',
  rum: 'ro',
  greek: 'el',
  gre: 'el',
  ell: 'el',
}

/** Last token of a subtitle name, ignoring flags like forced/sdh. */
function languageToken(name: string): string {
  const base = stripExtension(name).toLowerCase()
  const parts = base.split(/[._\-\s]+/).filter(Boolean)
  const flags = new Set(['forced', 'sdh', 'cc', 'full', 'default', 'sub', 'subs'])
  for (let index = parts.length - 1; index >= 0; index -= 1) {
    const part = parts[index]
    if (flags.has(part)) continue
    if (LANGUAGE_NAMES[part] || ISO_CODES[part]) return part
  }
  return ''
}

export function subtitleLanguage(name: string): { code: string; label: string } {
  const token = languageToken(name)
  if (!token) return { code: 'und', label: 'Subtitles' }
  return {
    code: ISO_CODES[token] ?? (token.length <= 3 ? token : 'und'),
    label: LANGUAGE_NAMES[token] ?? token.toUpperCase(),
  }
}

export function subtitleLabel(track: SubtitleTrack): string {
  const language = subtitleLanguage(track.name)
  const lower = track.name.toLowerCase()
  const flags: string[] = []
  if (lower.includes('forced')) flags.push('Forced')
  if (lower.includes('sdh') || lower.includes('cc')) flags.push('SDH')
  return flags.length ? `${language.label} (${flags.join(', ')})` : language.label
}

export function subtitleUrl(torrentId: number, fileIndex: number): string {
  return `/api/subtitles/${torrentId}/${fileIndex}`
}

/** Original subtitle bytes (ASS/SSA/VTT/SRT) for client-side rendering. */
export function rawSubtitleUrl(torrentId: number, fileIndex: number): string {
  return `/api/subtitles/${torrentId}/${fileIndex}?format=raw`
}

export function subtitleExtension(name: string): string {
  return name.split('/').pop()?.split('.').pop()?.toLowerCase() ?? ''
}

function ensureVtt(text: string): string {
  const body = text.trimStart()
  return body.startsWith('WEBVTT') ? text : `WEBVTT\n\n${text}`
}

function srtToVtt(text: string): string {
  const body = text
    .replace(/\r\n/g, '\n')
    .replace(/\r/g, '\n')
    .split('\n')
    .map(line => (line.includes('-->') ? line.replace(/,/g, '.') : line))
    .join('\n')
  return `WEBVTT\n\n${body.trim()}`
}

function assTime(value: string): string | null {
  const pieces = value.trim().split(':')
  if (pieces.length !== 3) return null
  const hours = Number(pieces[0])
  const minutes = Number(pieces[1])
  const [secondsRaw, fractionRaw = ''] = pieces[2].split('.')
  const seconds = Number(secondsRaw)
  if (![hours, minutes, seconds].every(Number.isFinite)) return null
  const fraction = `${fractionRaw}000`.slice(0, 3)
  return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}:${String(
    seconds,
  ).padStart(2, '0')}.${fraction}`
}

function stripAssTags(text: string): string {
  return text
    .replace(/\{[^}]*\}/g, '')
    .replace(/\\N/g, '\n')
    .replace(/\\n/g, '\n')
    .replace(/\\h/g, ' ')
}

function assToVtt(text: string): string {
  const fields = [
    'Layer',
    'Start',
    'End',
    'Style',
    'Name',
    'MarginL',
    'MarginR',
    'MarginV',
    'Effect',
    'Text',
  ]
  const cues: string[] = []
  let inEvents = false

  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trimEnd()
    if (line.startsWith('[') && line.endsWith(']')) {
      inEvents = line.toLowerCase() === '[events]'
      continue
    }
    if (!inEvents) continue
    if (line.startsWith('Format:')) {
      const parsed = line
        .slice('Format:'.length)
        .split(',')
        .map(field => field.trim())
      if (parsed.length) fields.splice(0, fields.length, ...parsed)
      continue
    }
    if (!line.startsWith('Dialogue:')) continue

    const parts = line.slice('Dialogue:'.length).split(',')
    const values = parts.slice(0, Math.max(0, fields.length - 1))
    values.push(parts.slice(Math.max(0, fields.length - 1)).join(','))
    const value = (name: string) => {
      const index = fields.findIndex(field => field.toLowerCase() === name.toLowerCase())
      return index >= 0 ? (values[index] ?? '').trim() : ''
    }
    const start = assTime(value('Start'))
    const end = assTime(value('End'))
    if (!start || !end) continue
    const cue = stripAssTags(value('Text')).trim()
    if (!cue) continue
    cues.push(`${start} --> ${end}\n${cue}`)
  }

  return `WEBVTT\n\n${cues.join('\n\n')}`
}

/** Convert any supported subtitle file to WebVTT for native `<track>` use. */
export function subtitleToVtt(name: string, content: string): string {
  switch (subtitleExtension(name)) {
    case 'vtt':
      return ensureVtt(content)
    case 'srt':
      return srtToVtt(content)
    case 'ass':
    case 'ssa':
      return assToVtt(content)
    default:
      return ensureVtt(content)
  }
}

function episodeKey(name: string): string | null {
  const match = name.match(/s(\d{1,2})e(\d{1,3})/i)
  return match ? `s${Number(match[1])}e${Number(match[2])}` : null
}

/** Pick subtitles that belong to the currently playing file. */
export function matchingSubtitles(videoName: string, tracks: SubtitleTrack[]): SubtitleTrack[] {
  if (tracks.length === 0) return []
  const videoEpisode = episodeKey(videoName)
  const videoBase = stripExtension(videoName).toLowerCase()

  const matches = tracks.filter(track => {
    const trackEpisode = episodeKey(track.name)
    if (videoEpisode && trackEpisode) return videoEpisode === trackEpisode
    const trackBase = stripExtension(track.name)
      .toLowerCase()
      .replace(/[._\-\s]+(forced|sdh|cc|full|default)$/i, '')
    return (
      trackBase.startsWith(videoBase) ||
      videoBase.startsWith(trackBase) ||
      trackBase.includes(videoBase) ||
      videoBase.includes(trackBase)
    )
  })

  return matches
}
