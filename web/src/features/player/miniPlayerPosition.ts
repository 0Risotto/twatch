const MINI_WIDTH = 340

export function miniWidth(): number {
  return Math.min(MINI_WIDTH, Math.max(200, window.innerWidth - 32))
}

export function clampMiniPosition(x: number, y: number): { x: number; y: number } {
  const width = miniWidth()
  const height = (width * 9) / 16
  const maxX = Math.max(8, window.innerWidth - width - 8)
  const maxY = Math.max(8, window.innerHeight - height - 8)
  return {
    x: Math.min(Math.max(8, x), maxX),
    y: Math.min(Math.max(8, y), maxY),
  }
}

/** Default to the bottom-right corner, above mobile navigation when present. */
export function defaultMiniPosition(): { x: number; y: number } {
  const width = miniWidth()
  const height = (width * 9) / 16
  const bottom = window.innerWidth < 640 ? 80 : 16
  return clampMiniPosition(window.innerWidth - width - 16, window.innerHeight - height - bottom)
}
