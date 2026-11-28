import { describe, expect, it } from 'vitest'

import {
  isSubtitleFile,
  matchingSubtitles,
  rawSubtitleUrl,
  subtitleExtension,
  subtitleLabel,
  subtitleLanguage,
  subtitleToVtt,
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
    expect(subtitleLanguage('episode.ptbr.srt')).toEqual({
      code: 'pt-BR',
      label: 'Portuguese (BR)',
    })
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

  it('builds subtitle endpoint URLs', () => {
    expect(subtitleUrl(7, 3)).toBe('/api/subtitles/7/3')
    expect(rawSubtitleUrl(7, 3)).toBe('/api/subtitles/7/3?format=raw')
    expect(subtitleExtension('subs/Show.S01E02.ass')).toBe('ass')
  })

  it('converts SRT and ASS files to WebVTT', () => {
    const srt = '1\r\n00:00:01,000 --> 00:00:02,500\r\nHello\r\n'
    expect(subtitleToVtt('movie.srt', srt)).toBe(
      'WEBVTT\n\n1\n00:00:01.000 --> 00:00:02.500\nHello',
    )

    const ass = `[Script Info]
ScriptType: v4.00+

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:03.50,Default,,0,0,0,,{\\an8}Top text
Dialogue: 0,0:00:04.00,0:00:05.25,Default,,0,0,0,,{\\i1}Italic, with comma
`
    expect(subtitleToVtt('movie.ass', ass)).toBe(
      'WEBVTT\n\n00:00:01.000 --> 00:00:03.500\nTop text\n\n00:00:04.000 --> 00:00:05.250\nItalic, with comma',
    )
    expect(subtitleToVtt('movie.vtt', 'WEBVTT\n\nHello')).toBe('WEBVTT\n\nHello')
  })
})
