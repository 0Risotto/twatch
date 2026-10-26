import { useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useUiStore } from '@/store/ui'

function isTyping(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  const tag = target.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || target.isContentEditable
}

/** Global keyboard shortcuts: command palette, help, and `g` navigation. */
export function useGlobalShortcuts(): void {
  const navigate = useNavigate()
  const setCommandOpen = useUiStore(state => state.setCommandOpen)
  const setShortcutsOpen = useUiStore(state => state.setShortcutsOpen)

  useEffect(() => {
    let pendingG = 0
    const onKeyDown = (event: KeyboardEvent) => {
      const meta = event.metaKey || event.ctrlKey
      if (meta && event.key.toLowerCase() === 'k') {
        event.preventDefault()
        setCommandOpen(true)
        return
      }
      if (isTyping(event.target)) return

      if (event.key === '?') {
        event.preventDefault()
        setShortcutsOpen(true)
        return
      }
      if (event.key === 'g') {
        pendingG = Date.now()
        return
      }
      if (pendingG && Date.now() - pendingG < 700) {
        pendingG = 0
        const key = event.key.toLowerCase()
        const routes: Record<string, string> = {
          h: '/',
          s: '/search',
          d: '/downloads',
          p: '/player',
          y: '/history',
          t: '/settings',
        }
        const destination = routes[key]
        if (destination) {
          event.preventDefault()
          navigate(destination)
        }
      }
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [navigate, setCommandOpen, setShortcutsOpen])
}
