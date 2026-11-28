import { BrowserRouter, Route, Routes } from 'react-router-dom'

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MotionConfig } from 'motion/react'

import { useUiStore } from '@/store/ui'

import { TooltipProvider } from '@/components/ui/Tooltip'

import { PlaybackProvider } from '@/features/player/PlaybackProvider'

import { DownloadsPage } from '@/pages/DownloadsPage'
import { HistoryPage } from '@/pages/HistoryPage'
import { HomePage } from '@/pages/HomePage'
import { LibraryPage } from '@/pages/LibraryPage'
import { NotFoundPage } from '@/pages/NotFoundPage'
import { PlayerPage } from '@/pages/PlayerPage'
import { SearchPage } from '@/pages/SearchPage'
import { SettingsPage } from '@/pages/SettingsPage'
import { TorrentPage } from '@/pages/TorrentPage'

import { Shell } from './Shell'
import { ToastViewport } from './ToastViewport'
import { useLive, useThemeSync } from './useLive'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
      staleTime: 10_000,
    },
  },
})

function AppShell() {
  useLive()
  useThemeSync()
  const motion = useUiStore(state => state.motion)

  return (
    <MotionConfig reducedMotion={motion === 'reduced' ? 'always' : 'user'}>
      <BrowserRouter>
        <PlaybackProvider>
          <Routes>
            <Route element={<Shell />}>
              <Route index element={<HomePage />} />
              <Route path="search" element={<SearchPage />} />
              <Route path="torrent" element={<TorrentPage />} />
              <Route path="downloads" element={<DownloadsPage />} />
              <Route path="library" element={<LibraryPage />} />
              <Route path="player" element={<PlayerPage />} />
              <Route path="history" element={<HistoryPage />} />
              <Route path="settings" element={<SettingsPage />} />
              <Route path="*" element={<NotFoundPage />} />
            </Route>
          </Routes>
        </PlaybackProvider>
      </BrowserRouter>
      <ToastViewport />
    </MotionConfig>
  )
}

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider delayDuration={350}>
        <AppShell />
      </TooltipProvider>
    </QueryClientProvider>
  )
}
