import { describe, expect, it } from 'vitest'

import { clampMiniPosition, defaultMiniPosition, miniWidth } from './miniPlayerPosition'

function setViewport(width: number, height: number) {
  Object.defineProperty(window, 'innerWidth', { configurable: true, value: width })
  Object.defineProperty(window, 'innerHeight', { configurable: true, value: height })
}

describe('mini player position', () => {
  it('defaults to the bottom-right corner on desktop', () => {
    setViewport(1280, 720)
    expect(miniWidth()).toBe(340)
    expect(defaultMiniPosition()).toEqual({ x: 924, y: 512.75 })
  })

  it('leaves room for mobile navigation', () => {
    setViewport(390, 844)
    expect(defaultMiniPosition()).toEqual({ x: 34, y: 572.75 })
  })

  it('clamps dragged positions inside the viewport', () => {
    setViewport(1280, 720)
    expect(clampMiniPosition(-500, -500)).toEqual({ x: 8, y: 8 })
    expect(clampMiniPosition(2000, 2000)).toEqual({ x: 932, y: 520.75 })
  })
})
