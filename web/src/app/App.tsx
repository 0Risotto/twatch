import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import { MotionConfig } from 'motion/react'
import { TooltipProvider } from '@/components/ui/Tooltip'
import { ToastViewport } from '@/components/ui/Toast'
import { useUiStore } from '@/store/ui'
import { Shell } from './Shell'
import { useLive, useThemeSync } from './useLive'
import { HomePage } from '@/pages/HomePage'
import { SearchPage } from '@/pages/SearchPage'
import { TorrentPage } from '@/pages/TorrentPage'
import { DownloadsPage } from '@/pages/DownloadsPage'
import { PlayerPage } from '@/pages/PlayerPage'
import { HistoryPage } from '@/pages/HistoryPage'
import { SettingsPage } from '@/pages/SettingsPage'
import { NotFoundPage } from '@/pages/NotFoundPage'

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
        <Routes>
          <Route element={<Shell />}>
            <Route index element={<HomePage />} />
            <Route path="search" element={<SearchPage />} />
            <Route path="torrent" element={<TorrentPage />} />
            <Route path="downloads" element={<DownloadsPage />} />
            <Route path="player" element={<PlayerPage />} />
            <Route path="history" element={<HistoryPage />} />
            <Route path="settings" element={<SettingsPage />} />
            <Route path="*" element={<NotFoundPage />} />
          </Route>
        </Routes>
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
