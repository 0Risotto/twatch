import { describe, expect, it } from 'vitest'
import {
  formatBytes,
  formatEta,
  formatPercent,
  formatPlaybackTimes,
  formatTime,
  isTorrentInput,
  magnetFor,
  pluralize,
} from './format'

describe('format helpers', () => {
  it('formats bytes across units', () => {
    expect(formatBytes(0)).toBe('0 B')
    expect(formatBytes(512)).toBe('512 B')
    expect(formatBytes(1024)).toBe('1.0 KB')
    expect(formatBytes(5 * 1024 * 1024)).toBe('5.0 MB')
  })

  it('formats eta values', () => {
    expect(formatEta(null)).toBe('—')
    expect(formatEta(12)).toBe('12s')
    expect(formatEta(125)).toBe('2m 5s')
    expect(formatEta(3725)).toBe('1h 2m')
    expect(formatEta(90000)).toBe('1d 1h')
  })

  it('clamps progress percentages', () => {
    expect(formatPercent(-1)).toBe('0%')
    expect(formatPercent(0.456)).toBe('46%')
    expect(formatPercent(2)).toBe('100%')
  })

  it('formats media timestamps', () => {
    expect(formatTime(0)).toBe('0:00')
    expect(formatTime(65)).toBe('1:05')
    expect(formatTime(3725)).toBe('1:02:05')
    expect(formatTime(Number.NaN)).toBe('0:00')
  })

  it('formats elapsed and remaining playback time', () => {
    expect(formatPlaybackTimes(83, 551)).toEqual({ elapsed: '1:23', remaining: '7:48' })
    expect(formatPlaybackTimes(600, 600)).toEqual({ elapsed: '10:00', remaining: '0:00' })
    expect(formatPlaybackTimes(10, 0)).toEqual({ elapsed: '0:10', remaining: '0:00' })
  })

  it('distinguishes torrent sources from search titles', () => {
    expect(isTorrentInput('magnet:?xt=urn:btih:abc')).toBe(true)
    expect(isTorrentInput('https://example.com/file.torrent')).toBe(true)
    expect(isTorrentInput('Dune Part Two')).toBe(false)
    expect(isTorrentInput('jujutsu kaisen s02')).toBe(false)
  })

  it('builds a magnet and pluralizes', () => {
    expect(
      magnetFor({ name: 'A B', infoHash: 'abc', seeders: 1, leechers: 2, size: 3, username: 'u' }),
    ).toBe('magnet:?xt=urn:btih:abc&dn=A%20B')
    expect(pluralize(1, 'file')).toBe('1 file')
    expect(pluralize(2, 'file')).toBe('2 files')
  })
})
