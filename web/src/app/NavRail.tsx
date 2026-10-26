import { motion } from 'motion/react'
import {
  ArrowDownToLine,
  Clock3,
  Home,
  Play,
  Search,
  Settings,
} from 'lucide-react'
import { NavLink, useLocation } from 'react-router-dom'
import { cn } from '@/lib/cn'
import { springs } from '@/lib/motion'
import { Tooltip } from '@/components/ui/Tooltip'
import { useAppStore } from '@/store/app'

export const NAV_ITEMS = [
  { to: '/', label: 'Home', icon: Home, match: (path: string) => path === '/' },
  { to: '/search', label: 'Search', icon: Search, match: (path: string) => path.startsWith('/search') },
  { to: '/downloads', label: 'Downloads', icon: ArrowDownToLine, match: (path: string) => path.startsWith('/downloads') },
  { to: '/player', label: 'Player', icon: Play, match: (path: string) => path.startsWith('/player') },
  { to: '/history', label: 'History', icon: Clock3, match: (path: string) => path.startsWith('/history') },
  { to: '/settings', label: 'Settings', icon: Settings, match: (path: string) => path.startsWith('/settings') },
] as const

function Brand() {
  // Raw supplied mark: no tile, gradient, border, or shadow around it.
  return <img src="/tv_ghost_logo.svg" alt="twatch" className="size-10 light:invert" />
}

export function NavRail() {
  const location = useLocation()
  const activeDownloads = useAppStore(
    state => state.downloads.filter(download => download.state !== 'completed' && download.state !== 'failed').length,
  )

  return (
    <aside className="sticky top-0 z-20 hidden h-screen w-[84px] shrink-0 self-start flex-col items-center border-r border-[var(--border-soft)] bg-[var(--bg)] py-4 backdrop-blur-xl md:flex">
      <NavLink to="/" aria-label="twatch home" className="mb-5">
        <Brand />
      </NavLink>

      <nav className="flex flex-1 flex-col items-center gap-1" aria-label="Main navigation">
        {NAV_ITEMS.map(item => {
          const active = item.match(location.pathname)
          const Icon = item.icon
          const showCount = item.to === '/downloads' && activeDownloads > 0
          return (
            <Tooltip key={item.to} content={item.label} side="right">
              <NavLink
                to={item.to}
                className={cn(
                  'relative flex h-[54px] w-[64px] flex-col items-center justify-center gap-1 rounded-xl text-[10px] font-semibold tracking-wide transition-colors',
                  active ? 'text-[var(--fg)]' : 'text-[var(--muted)] hover:text-[var(--fg)]',
                )}
              >
                {active && (
                  <motion.span
                    layoutId="nav-active"
                    className="absolute inset-0 rounded-xl border border-[var(--border-soft)] bg-[var(--surface-2)] shadow-[var(--shadow-sm)]"
                    transition={springs.layout}
                  />
                )}
                <span className="relative z-10 grid place-items-center">
                  <Icon className="size-[18px]" strokeWidth={active ? 2.2 : 1.8} />
                  {showCount && (
                    <span className="absolute -right-2.5 -top-2 grid size-4 place-items-center rounded-full bg-[var(--accent)] text-[9px] font-bold text-[var(--accent-fg)]">
                      {activeDownloads > 9 ? '9+' : activeDownloads}
                    </span>
                  )}
                </span>
                <span className="relative z-10">{item.label}</span>
              </NavLink>
            </Tooltip>
          )
        })}
      </nav>

      <div className="flex flex-col items-center pt-2">
        <span className="font-mono text-[9px] text-[var(--faint)]">v0.1</span>
      </div>
    </aside>
  )
}

export function MobileNav() {
  const location = useLocation()
  const activeDownloads = useAppStore(
    state => state.downloads.filter(download => download.state !== 'completed' && download.state !== 'failed').length,
  )
  return (
    <nav
      className="glass fixed inset-x-3 bottom-3 z-40 flex items-center justify-around rounded-[20px] px-1.5 py-1.5 md:hidden"
      aria-label="Main navigation"
    >
      {NAV_ITEMS.map(item => {
        const active = item.match(location.pathname)
        const Icon = item.icon
        return (
          <NavLink
            key={item.to}
            to={item.to}
            className={cn(
              'relative flex size-12 flex-col items-center justify-center gap-0.5 rounded-[14px] text-[9px] font-semibold',
              active ? 'text-[var(--fg)]' : 'text-[var(--muted)]',
            )}
          >
            {active && (
              <motion.span
                layoutId="mobile-nav-active"
                className="absolute inset-0 rounded-[14px] bg-[var(--surface-2)]"
                transition={springs.layout}
              />
            )}
            <Icon className="relative z-10 size-[18px]" />
            <span className="relative z-10">{item.label}</span>
            {item.to === '/downloads' && activeDownloads > 0 && (
              <span className="absolute right-1.5 top-1 size-1.5 rounded-full bg-[var(--accent)]" />
            )}
          </NavLink>
        )
      })}
    </nav>
  )
}
