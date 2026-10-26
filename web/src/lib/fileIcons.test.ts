import { describe, expect, it } from 'vitest'
import { canPreview, extensionOf, fileKind, mediaKind, previewLabel } from './fileIcons'

describe('file icons', () => {
  it('reads extensions', () => {
    expect(extensionOf('folder/Movie.2024.MKV')).toBe('mkv')
    expect(extensionOf('no-extension')).toBe('')
    expect(extensionOf('.hidden')).toBe('')
  })

  it('classifies media and documents', () => {
    expect(fileKind('Movie.mkv')).toBe('movie')
    expect(fileKind('Show.S01E02.mkv')).toBe('tv')
    expect(fileKind('Track.flac')).toBe('audio')
    expect(fileKind('Cover.PNG')).toBe('image')
    expect(fileKind('Subs.srt')).toBe('subtitle')
    expect(fileKind('Book.epub')).toBe('ebook')
    expect(fileKind('Disk.iso')).toBe('disc')
    expect(fileKind('Archive.7z')).toBe('archive')
    expect(fileKind('main.rs')).toBe('code')
  })

  it('only previews video, audio and images', () => {
    expect(canPreview('movie')).toBe(true)
    expect(canPreview('tv')).toBe(true)
    expect(canPreview('audio')).toBe(true)
    expect(canPreview('image')).toBe(true)
    expect(canPreview('pdf')).toBe(false)
    expect(canPreview('archive')).toBe(false)
    expect(previewLabel('movie')).toBe('Watch video')
    expect(previewLabel('audio')).toBe('Play audio')
    expect(previewLabel('image')).toBe('Preview image')
  })

  it('guesses torrent titles without extensions', () => {
    expect(mediaKind('Perfume - The Story Of A Murderer (2006)')).toBe('movie')
    expect(mediaKind('Planet Earth S01E02 2160p')).toBe('tv')
    expect(mediaKind('Modern Family S01-S08 (2009-)')).toBe('tv')
    expect(mediaKind('A Book About Things EPUB')).toBe('ebook')
    expect(mediaKind('Some Album FLAC')).toBe('audio')
    expect(mediaKind('A Repack GOG')).toBe('game')
    expect(mediaKind('Unknown Release')).toBe('other')
  })
})
