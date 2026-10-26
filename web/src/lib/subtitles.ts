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
  en: 'English', eng: 'English', english: 'English',
  es: 'Spanish', spa: 'Spanish', esp: 'Spanish',
  fr: 'French', fre: 'French', fra: 'French',
  de: 'German', ger: 'German', deu: 'German',
  it: 'Italian', ita: 'Italian',
  pt: 'Portuguese', por: 'Portuguese', ptbr: 'Portuguese (BR)', pb: 'Portuguese (BR)',
  nl: 'Dutch', dut: 'Dutch', nld: 'Dutch',
  pl: 'Polish', pol: 'Polish',
  ru: 'Russian', rus: 'Russian',
  uk: 'Ukrainian', ukr: 'Ukrainian',
  ja: 'Japanese', jpn: 'Japanese', jp: 'Japanese',
  ko: 'Korean', kor: 'Korean',
  zh: 'Chinese', chi: 'Chinese', zho: 'Chinese', chs: 'Chinese (Simplified)', cht: 'Chinese (Traditional)',
  ar: 'Arabic', ara: 'Arabic',
  he: 'Hebrew', heb: 'Hebrew',
  hi: 'Hindi', hin: 'Hindi',
  ta: 'Tamil', tam: 'Tamil',
  te: 'Telugu', tel: 'Telugu',
  th: 'Thai', tha: 'Thai',
  vi: 'Vietnamese', vie: 'Vietnamese',
  id: 'Indonesian', ind: 'Indonesian',
  ms: 'Malay', may: 'Malay',
  tr: 'Turkish', tur: 'Turkish',
  sv: 'Swedish', swe: 'Swedish',
  da: 'Danish', dan: 'Danish',
  fi: 'Finnish', fin: 'Finnish',
  no: 'Norwegian', nor: 'Norwegian',
  cs: 'Czech', ces: 'Czech', cze: 'Czech',
  hu: 'Hungarian', hun: 'Hungarian',
  ro: 'Romanian', ron: 'Romanian', rum: 'Romanian',
  el: 'Greek', gre: 'Greek', ell: 'Greek',
}

const ISO_CODES: Record<string, string> = {
  english: 'en', eng: 'en', spanish: 'es', spa: 'es', esp: 'es',
  french: 'fr', fre: 'fr', fra: 'fr', german: 'de', ger: 'de', deu: 'de',
  italian: 'it', ita: 'it', portuguese: 'pt', por: 'pt', ptbr: 'pt-BR',
  dutch: 'nl', dut: 'nl', nld: 'nl', polish: 'pl', pol: 'pl',
  russian: 'ru', rus: 'ru', ukrainian: 'uk', ukr: 'uk', japanese: 'ja',
  jpn: 'ja', jp: 'ja', korean: 'ko', kor: 'ko', chinese: 'zh', chi: 'zh',
  zho: 'zh', chs: 'zh-CN', cht: 'zh-TW', arabic: 'ar', ara: 'ar',
  hebrew: 'he', heb: 'he', hindi: 'hi', hin: 'hi', tamil: 'ta', tam: 'ta',
  telugu: 'te', tel: 'te', thai: 'th', tha: 'th', vietnamese: 'vi', vie: 'vi',
  indonesian: 'id', ind: 'id', malay: 'ms', may: 'ms', turkish: 'tr', tur: 'tr',
  swedish: 'sv', swe: 'sv', danish: 'da', dan: 'da', finnish: 'fi', fin: 'fi',
  norwegian: 'no', nor: 'no', czech: 'cs', ces: 'cs', cze: 'cs',
  hungarian: 'hu', hun: 'hu', romanian: 'ro', ron: 'ro', rum: 'ro',
  greek: 'el', gre: 'el', ell: 'el',
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

function episodeKey(name: string): string | null {
  const match = name.match(/s(\d{1,2})e(\d{1,3})/i)
  return match ? `s${Number(match[1])}e${Number(match[2])}` : null
}

/** Pick subtitles that belong to the currently playing file. */
export function matchingSubtitles(
  videoName: string,
  tracks: SubtitleTrack[],
): SubtitleTrack[] {
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
