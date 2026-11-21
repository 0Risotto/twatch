import { useEffect } from 'react'

interface ShortcutActions {
  enabled: boolean
  toggle: () => void
  seek: (time: number) => void
  skip: (delta: number) => void
  setVolume: (volume: number) => void
  volume: () => number
  toggleMute: () => void
  toggleFullscreen: () => void
  togglePip: () => void
  cycleSubtitles: () => void
  next: () => void
  prev: () => void
  setRate: (rate: number) => void
  rate: () => number
}

function isInteractive(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  return Boolean(
    target.closest('input, textarea, select, [contenteditable="true"], [role="slider"]'),
  )
}

/**
 * Player-scoped keyboard controls.
 *
 * Capture phase + `stopPropagation` keeps the global `g`/`?` bindings from
 * firing while the player is focused.
 */
export function usePlayerShortcuts(actions: ShortcutActions): void {
  useEffect(() => {
    if (!actions.enabled) return

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.metaKey || event.ctrlKey || event.altKey) return
      if (isInteractive(event.target)) return

      const key = event.key
      let handled = false

      switch (key) {
        case ' ':
        case 'k':
        case 'K':
          actions.toggle()
          handled = true
          break
        case 'ArrowLeft':
          actions.skip(event.shiftKey ? -30 : -5)
          handled = true
          break
        case 'ArrowRight':
          actions.skip(event.shiftKey ? 30 : 5)
          handled = true
          break
        case 'ArrowUp':
          actions.setVolume(actions.volume() + 0.05)
          handled = true
          break
        case 'ArrowDown':
          actions.setVolume(actions.volume() - 0.05)
          handled = true
          break
        case 'j':
        case 'J':
          actions.skip(-10)
          handled = true
          break
        case 'l':
        case 'L':
          actions.skip(10)
          handled = true
          break
        case 's':
        case 'S':
          actions.skip(85)
          handled = true
          break
        case 'r':
        case 'R':
          actions.skip(-10)
          handled = true
          break
        case 'm':
        case 'M':
          actions.toggleMute()
          handled = true
          break
        case 'f':
        case 'F':
          actions.toggleFullscreen()
          handled = true
          break
        case 'p':
        case 'P':
          actions.togglePip()
          handled = true
          break
        case 'c':
        case 'C':
          actions.cycleSubtitles()
          handled = true
          break
        case 'n':
        case 'N':
          actions.next()
          handled = true
          break
        case 'b':
        case 'B':
          actions.prev()
          handled = true
          break
        case '[':
          actions.setRate(actions.rate() - 0.25)
          handled = true
          break
        case ']':
          actions.setRate(actions.rate() + 0.25)
          handled = true
          break
        case '\\':
          actions.setRate(1)
          handled = true
          break
        case 'Home':
          actions.seek(0)
          handled = true
          break
        default:
          break
      }

      if (handled) {
        event.preventDefault()
        event.stopPropagation()
      }
    }

    window.addEventListener('keydown', onKeyDown, true)
    return () => window.removeEventListener('keydown', onKeyDown, true)
  }, [actions])
}
