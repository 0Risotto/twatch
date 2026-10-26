import { describe, expect, it } from 'vitest'
import {
  isSubtitleFile,
  matchingSubtitles,
  subtitleLabel,
  subtitleLanguage,
  subtitleUrl,
} from './subtitles'

describe('subtitles', () => {
  it('recognises browser-convertible subtitle files', () => {
    expect(isSubtitleFile('episode.srt')).toBe(true)
    expect(isSubtitleFile('Episode.ASS')).toBe(true)
    expect(isSubtitleFile('subs/Episode_en.vtt')).toBe(true)
    expect(isSubtitleFile('episode.mkv')).toBe(false)
    expect(isSubtitleFile('notes.txt')).toBe(false)
  })

  it('detects languages and labels', () => {
    expect(subtitleLanguage('episode.en.srt')).toEqual({ code: 'en', label: 'English' })
    expect(subtitleLanguage('episode.ptbr.srt')).toEqual({ code: 'pt-BR', label: 'Portuguese (BR)' })
    expect(subtitleLanguage('episode.srt')).toEqual({ code: 'und', label: 'Subtitles' })
    expect(subtitleLabel({ index: 2, name: 'episode.en.forced.srt' })).toBe('English (Forced)')
    expect(subtitleLabel({ index: 2, name: 'episode.en.sdh.srt' })).toBe('English (SDH)')
  })

  it('matches subtitle files to the playing episode', () => {
    const tracks = [
      { index: 2, name: 'Show.S01E02.en.srt' },
      { index: 3, name: 'Show.S01E03.en.srt' },
      { index: 4, name: 'Show.S01E02.spa.srt' },
    ]
    const matched = matchingSubtitles('Show.S01E02.1080p.mkv', tracks)
    expect(matched.map(track => track.index)).toEqual([2, 4])
  })

  it('falls back to base-name matching', () => {
    const tracks = [
      { index: 2, name: 'Movie.en.srt' },
      { index: 3, name: 'Other.en.srt' },
    ]
    expect(matchingSubtitles('Movie.mkv', tracks).map(track => track.index)).toEqual([2])
  })

  it('builds the subtitle endpoint URL', () => {
    expect(subtitleUrl(7, 3)).toBe('/api/subtitles/7/3')
  })
})
