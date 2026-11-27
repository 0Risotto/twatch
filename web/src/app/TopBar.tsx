import { useMemo } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { Command, Plus, Search } from 'lucide-react'
import { Button, IconButton } from '@/components/ui/Button'
import { useAppStore } from '@/store/app'
import { useUiStore } from '@/store/ui'
import { commandKeyLabel } from '@/lib/platform'

const titles: Record<string, string> = {
  '/': 'Home',
  '/search': 'Search',
  '/torrent': 'Torrent',
  '/downloads': 'Downloads',
  '/library': 'Library',
  '/player': 'Player',
  '/history': 'History',
  '/settings': 'Settings',
}

export function TopBar() {
  const location = useLocation()
  const navigate = useNavigate()
  const downloads = useAppStore(state => state.downloads)
  const session = useAppStore(state => state.session)
  const setCommandOpen = useUiStore(state => state.setCommandOpen)

  const title = titles[location.pathname] ?? 'twatch'
  const totalSpeed = useMemo(
    () =>
      downloads
        .filter(download => download.state === 'downloading' || download.state === 'streaming')
        .reduce((sum, download) => sum + download.speed, 0),
    [downloads],
  )

  return (
    <header className="glass sticky top-0 z-30 flex h-[64px] items-center gap-3 border-x-0 border-t-0 px-4 sm:px-6 lg:px-8">
      <button
        type="button"
        aria-label="twatch home"
        onClick={() => navigate('/')}
        className="flex items-center gap-2.5 rounded-lg text-left transition hover:opacity-80 md:hidden"
      >
        <img src="/tv_ghost_logo.svg" alt="" className="size-8 light:invert" />
        <span className="font-semibold tracking-[-0.03em]">twatch</span>
      </button>

      <div className="hidden min-w-0 items-baseline gap-2 sm:flex">
        <span className="text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--faint)]">
          twatch
        </span>
        <span className="text-[var(--faint)]">/</span>
        <h1 className="truncate text-[15px] font-semibold tracking-[-0.02em] text-[var(--fg)]">
          {title}
        </h1>
        {session && location.pathname === '/torrent' && (
          <span className="hidden max-w-[260px] truncate text-xs text-[var(--muted)] lg:inline">
            {session.name}
          </span>
        )}
      </div>

      <button
        type="button"
        onClick={() => setCommandOpen(true)}
        className="group ml-auto hidden h-9 w-[220px] items-center gap-2 rounded-[3px] border border-[var(--border-soft)] bg-[color-mix(in_srgb,var(--surface)_55%,transparent)] px-3 text-left text-xs text-[var(--muted)] transition hover:border-[var(--border-strong)] hover:text-[var(--fg)] sm:flex lg:w-[280px]"
      >
        <Search className="size-3.5" />
        <span className="flex-1">Jump to…</span>
        <span className="kbd">{commandKeyLabel}</span>
      </button>

      <div className="ml-auto flex items-center gap-1.5 sm:ml-0">
        {totalSpeed > 0 && (
          <span className="hidden font-mono text-[11px] tabular text-[var(--muted)] lg:inline">
            ↓ {formatSpeed(totalSpeed)}
          </span>
        )}

        <IconButton
          label="Open command palette"
          onClick={() => setCommandOpen(true)}
          className="sm:hidden"
        >
          <Command className="size-4" />
        </IconButton>
        <Button
          variant="primary"
          size="sm"
          aria-label="Add torrent"
          onClick={() => navigate('/')}
          className="gap-1.5"
        >
          <Plus className="size-4" />
          <span className="hidden sm:inline">Add torrent</span>
        </Button>
      </div>
    </header>
  )
}

import { formatSpeed } from '@/lib/format'
