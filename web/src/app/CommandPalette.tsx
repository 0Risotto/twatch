import { useEffect, useMemo, useRef, useState } from 'react'
import { AnimatePresence, motion } from 'motion/react'
import * as Dialog from '@radix-ui/react-dialog'
import { useNavigate } from 'react-router-dom'
import {
  ArrowDownToLine,
  Clock3,
  Command,
  Home,
  Keyboard,
  Play,
  Search,
  Settings,
} from 'lucide-react'
import { cn } from '@/lib/cn'
import { fade, modalPanel } from '@/lib/motion'
import { useUiStore } from '@/store/ui'

interface Action {
  id: string
  label: string
  hint: string
  icon: typeof Home
  keywords: string
  run: () => void
}

export function CommandPalette() {
  const open = useUiStore(state => state.commandOpen)
  const setOpen = useUiStore(state => state.setCommandOpen)
  const setShortcutsOpen = useUiStore(state => state.setShortcutsOpen)
  const navigate = useNavigate()
  const [query, setQuery] = useState('')
  const [active, setActive] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)

  const actions = useMemo<Action[]>(() => {
    const go = (path: string) => () => {
      navigate(path)
      setOpen(false)
    }
    return [
      { id: 'home', label: 'Go home', hint: 'Add a torrent', icon: Home, keywords: 'home add magnet start', run: go('/') },
      { id: 'search', label: 'Search torrents', hint: 'Find a release', icon: Search, keywords: 'search find torrent', run: go('/search') },
      { id: 'downloads', label: 'Open downloads', hint: 'Live transfer activity', icon: ArrowDownToLine, keywords: 'downloads transfers progress', run: go('/downloads') },
      { id: 'player', label: 'Open player', hint: 'Now playing', icon: Play, keywords: 'player watch stream video', run: go('/player') },
      { id: 'history', label: 'Open history', hint: 'Revisit a torrent', icon: Clock3, keywords: 'history recent watched', run: go('/history') },
      { id: 'settings', label: 'Open settings', hint: 'Theme, search, playback', icon: Settings, keywords: 'settings theme config preferences', run: go('/settings') },
      {
        id: 'shortcuts',
        label: 'Keyboard shortcuts',
        hint: 'Show all shortcuts',
        icon: Keyboard,
        keywords: 'keyboard keys help shortcuts',
        run: () => {
          setOpen(false)
          setShortcutsOpen(true)
        },
      },
    ]
  }, [navigate, setOpen, setShortcutsOpen])

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase()
    if (!needle) return actions
    return actions.filter(action => `${action.label} ${action.hint} ${action.keywords}`.toLowerCase().includes(needle))
  }, [actions, query])

  useEffect(() => {
    if (open) {
      setQuery('')
      setActive(0)
      window.setTimeout(() => inputRef.current?.focus(), 30)
    }
  }, [open])

  useEffect(() => {
    setActive(0)
  }, [query])

  const onKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault()
      setActive(index => (index + 1) % Math.max(1, filtered.length))
    } else if (event.key === 'ArrowUp') {
      event.preventDefault()
      setActive(index => (index - 1 + Math.max(1, filtered.length)) % Math.max(1, filtered.length))
    } else if (event.key === 'Enter') {
      event.preventDefault()
      filtered[active]?.run()
    }
  }

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <AnimatePresence>
        {open && (
          <Dialog.Portal forceMount>
            <Dialog.Overlay asChild forceMount>
              <motion.div
                variants={fade}
                initial="initial"
                animate="animate"
                exit="exit"
                className="fixed inset-0 z-50 bg-[color-mix(in_srgb,var(--bg)_58%,transparent)] backdrop-blur-[4px]"
              />
            </Dialog.Overlay>
            <Dialog.Content asChild forceMount aria-describedby={undefined}>
              <motion.div
                variants={modalPanel}
                initial="initial"
                animate="animate"
                exit="exit"
                className="glass-strong fixed left-1/2 top-[14vh] z-50 w-[calc(100vw-2rem)] max-w-[620px] -translate-x-1/2 overflow-hidden rounded-[var(--radius-xl)]"
              >
                <Dialog.Title className="sr-only">Command palette</Dialog.Title>
                <div className="flex items-center gap-3 border-b border-[var(--border-soft)] px-4 py-3">
                  <Command className="size-4 shrink-0 text-[var(--accent)]" />
                  <input
                    ref={inputRef}
                    value={query}
                    onChange={event => setQuery(event.target.value)}
                    onKeyDown={onKeyDown}
                    placeholder="Type a command or jump to a page…"
                    className="h-6 w-full bg-transparent text-sm text-[var(--fg)] outline-none placeholder:text-[var(--faint)]"
                  />
                  <span className="kbd">esc</span>
                </div>
                <div className="max-h-[min(52vh,420px)] overflow-y-auto p-2">
                  {filtered.length === 0 && (
                    <p className="px-3 py-8 text-center text-xs text-[var(--muted)]">No commands match “{query}”.</p>
                  )}
                  {filtered.map((action, index) => {
                    const Icon = action.icon
                    return (
                      <button
                        key={action.id}
                        type="button"
                        onMouseEnter={() => setActive(index)}
                        onClick={action.run}
                        className={cn(
                          'flex w-full items-center gap-3 rounded-[10px] px-3 py-2.5 text-left transition-colors',
                          index === active ? 'bg-[var(--surface-2)]' : 'hover:bg-[var(--hover)]',
                        )}
                      >
                        <span className="grid size-8 place-items-center rounded-lg border border-[var(--border-soft)] bg-[var(--surface)] text-[var(--accent)]">
                          <Icon className="size-4" />
                        </span>
                        <span className="min-w-0 flex-1">
                          <span className="block text-sm font-medium text-[var(--fg)]">{action.label}</span>
                          <span className="block text-[11px] text-[var(--muted)]">{action.hint}</span>
                        </span>
                        {index === active && <span className="kbd">↵</span>}
                      </button>
                    )
                  })}
                </div>
                <div className="flex items-center gap-4 border-t border-[var(--border-soft)] px-4 py-2 text-[10px] text-[var(--faint)]">
                  <span className="flex items-center gap-1"><span className="kbd">↑↓</span> navigate</span>
                  <span className="flex items-center gap-1"><span className="kbd">↵</span> open</span>
                  <span className="ml-auto flex items-center gap-1"><span className="kbd">⌘K</span> toggle</span>
                </div>
              </motion.div>
            </Dialog.Content>
          </Dialog.Portal>
        )}
      </AnimatePresence>
    </Dialog.Root>
  )
}
