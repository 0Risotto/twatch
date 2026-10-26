import { AnimatePresence, motion } from 'motion/react'
import { Outlet, useLocation } from 'react-router-dom'
import { pageVariants } from '@/lib/motion'
import { NavRail, MobileNav } from './NavRail'
import { TopBar } from './TopBar'
import { ActivityPanel } from './ActivityPanel'
import { CommandPalette } from './CommandPalette'
import { ShortcutsHelp } from './ShortcutsHelp'
import { ErrorBoundary } from '@/components/ErrorBoundary'
import { useGlobalShortcuts } from './useGlobalShortcuts'

export function Shell() {
  const location = useLocation()
  useGlobalShortcuts()

  return (
    <div className="relative flex min-h-screen">
      <NavRail />
      <div className="relative z-10 flex min-w-0 flex-1 flex-col">
        <TopBar />
        <div className="mx-auto flex w-full max-w-[1600px] flex-1 gap-6 px-4 pb-28 pt-4 sm:px-6 lg:px-8 lg:pb-10">
          <main className="min-w-0 flex-1">
            <AnimatePresence mode="wait" initial={false}>
              <motion.div
                key={location.pathname}
                variants={pageVariants}
                initial="initial"
                animate="animate"
                exit="exit"
                className="min-h-full"
              >
                <ErrorBoundary key={location.pathname}>
                  <Outlet />
                </ErrorBoundary>
              </motion.div>
            </AnimatePresence>
          </main>
          <ActivityPanel />
        </div>
        <MobileNav />
      </div>
      <CommandPalette />
      <ShortcutsHelp />
    </div>
  )
}
