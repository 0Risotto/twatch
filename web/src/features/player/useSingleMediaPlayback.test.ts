import { renderHook } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { pauseOtherMedia, useSingleMediaPlayback } from './useSingleMediaPlayback'

afterEach(() => {
  document.body.replaceChildren()
})

describe('single media playback', () => {
  it('pauses all other media elements', () => {
    const first = document.createElement('video')
    const second = document.createElement('audio')
    const pauseFirst = vi.spyOn(first, 'pause').mockImplementation(() => {})
    const pauseSecond = vi.spyOn(second, 'pause').mockImplementation(() => {})
    document.body.append(first, second)

    pauseOtherMedia(second)

    expect(pauseFirst).toHaveBeenCalledOnce()
    expect(pauseSecond).not.toHaveBeenCalled()
  })

  it('ignores non-media targets', () => {
    const button = document.createElement('button')
    const pause = vi.spyOn(HTMLMediaElement.prototype, 'pause').mockImplementation(() => {})
    document.body.append(button)

    pauseOtherMedia(button)

    expect(pause).not.toHaveBeenCalled()
    pause.mockRestore()
  })

  it('pauses other media when a new element starts playing', () => {
    const { unmount } = renderHook(() => useSingleMediaPlayback())
    const first = document.createElement('video')
    const second = document.createElement('audio')
    const pauseFirst = vi.spyOn(first, 'pause').mockImplementation(() => {})
    const pauseSecond = vi.spyOn(second, 'pause').mockImplementation(() => {})
    document.body.append(first, second)

    second.dispatchEvent(new Event('playing'))

    expect(pauseFirst).toHaveBeenCalledOnce()
    expect(pauseSecond).not.toHaveBeenCalled()
    unmount()
  })
})
