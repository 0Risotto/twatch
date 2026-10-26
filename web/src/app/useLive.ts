import { useEffect } from 'react'
import { api } from '@/lib/api'
import { applyTheme, isKnownTheme } from '@/lib/themes'
import { useAppStore } from '@/store/app'
import type { WsFrame } from '@/types/api'

const MAX_RETRY_MS = 5000

/** Bootstrap the SPA and keep it in sync over the WebSocket. */
export function useLive(): void {
  useEffect(() => {
    let cancelled = false
    let socket: WebSocket | null = null
    let timer: number | undefined
    let attempts = 0

    const connect = async () => {
      try {
        const data = await api.bootstrap()
        if (cancelled) return
        useAppStore.getState().applyBootstrap(data)
      } catch (error) {
        if (cancelled) return
        useAppStore.getState().addToast('error', error instanceof Error ? error.message : 'Could not reach twatchd')
      }
      if (cancelled) return

      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
      socket = new WebSocket(`${protocol}//${window.location.host}/api/ws`)

      socket.onopen = () => {
        attempts = 0
        useAppStore.getState().setConnected(true)
        if (timer) window.clearTimeout(timer)
      }
      socket.onmessage = event => {
        try {
          const frame = JSON.parse(event.data as string) as WsFrame
          useAppStore.getState().applyFrame(frame)
        } catch {
          // Ignore malformed frames.
        }
      }
      socket.onerror = () => socket?.close()
      socket.onclose = () => {
        useAppStore.getState().setConnected(false)
        if (cancelled) return
        attempts += 1
        const delay = Math.min(MAX_RETRY_MS, 350 * 2 ** Math.min(attempts, 4))
        timer = window.setTimeout(connect, delay)
      }
    }

    void connect()
    return () => {
      cancelled = true
      if (timer) window.clearTimeout(timer)
      socket?.close()
    }
  }, [])
}

/** Apply the active theme from config/themes as CSS custom properties. */
export function useThemeSync(): void {
  const config = useAppStore(state => state.config)

  useEffect(() => {
    const stored = typeof localStorage === 'undefined' ? undefined : localStorage.getItem('twatch.theme') ?? undefined
    const candidate = config?.theme ?? stored ?? 'emerald-dark'
    applyTheme(isKnownTheme(candidate) ? candidate : 'emerald-dark')
  }, [config?.theme])
}
