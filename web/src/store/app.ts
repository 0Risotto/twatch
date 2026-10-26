import { create } from 'zustand'
import type {
  BootstrapSnapshot,
  ConfigView,
  DownloadSnapshot,
  PlayerSnapshot,
  SessionSnapshot,
  ThemeSpec,
  ToastItem,
  ToastLevel,
  WsFrame,
} from '@/types/api'

interface AppState {
  booted: boolean
  bootstrap: BootstrapSnapshot | null
  session: SessionSnapshot | null
  downloads: DownloadSnapshot[]
  player: PlayerSnapshot
  config: ConfigView | null
  themes: ThemeSpec[]
  searchConfigured: boolean
  connected: boolean
  revision: number
  toasts: ToastItem[]

  applyBootstrap: (data: BootstrapSnapshot) => void
  applyFrame: (frame: WsFrame) => void
  setConnected: (connected: boolean) => void
  setSession: (session: SessionSnapshot | null) => void
  setDownloads: (downloads: DownloadSnapshot[]) => void
  setPlayer: (player: PlayerSnapshot) => void
  setConfig: (config: ConfigView) => void
  setSearchConfigured: (configured: boolean) => void
  optimisticSelect: (indices: number[], selected: boolean) => void
  optimisticRemoveDownload: (id: number) => void
  optimisticRename: (index: number, name: string) => void
  addToast: (level: ToastLevel, message: string) => number
  dismissToast: (id: number) => void
}

let toastSeq = 0

const emptyPlayer: PlayerSnapshot = {
  running: false,
  title: null,
  streamPath: null,
  downloadId: null,
}

function recomputeSelection(session: SessionSnapshot, indices: number[], selected: boolean): SessionSnapshot {
  const wanted = new Set(indices)
  const files = session.files.map(file =>
    wanted.has(file.index) ? { ...file, selected } : file,
  )
  const selectedFiles = files.filter(file => file.selected)
  return {
    ...session,
    files,
    selectedCount: selectedFiles.length,
    selectedSize: selectedFiles.reduce((sum, file) => sum + file.size, 0),
  }
}

export const useAppStore = create<AppState>((set, get) => ({
  booted: false,
  bootstrap: null,
  session: null,
  downloads: [],
  player: emptyPlayer,
  config: null,
  themes: [],
  searchConfigured: false,
  connected: false,
  revision: 0,
  toasts: [],

  applyBootstrap: data =>
    set({
      booted: true,
      bootstrap: data,
      session: data.session,
      downloads: data.downloads,
      player: data.player,
      config: data.config,
      themes: data.themes,
      searchConfigured: data.searchConfigured,
      revision: 0,
    }),

  applyFrame: frame => {
    const state = get()
    if (frame.type === 'snapshot') {
      set({
        session: frame.session,
        downloads: frame.downloads,
        player: frame.player,
        revision: frame.revision,
        connected: true,
      })
      return
    }
    if (frame.revision <= state.revision) return
    switch (frame.type) {
      case 'session':
        set({ session: frame.session, revision: frame.revision })
        break
      case 'downloads':
        set({ downloads: frame.downloads, revision: frame.revision })
        break
      case 'player':
        set({ player: frame.player, revision: frame.revision })
        break
      case 'toast':
        set({
          revision: frame.revision,
          toasts: [
            ...state.toasts,
            { id: ++toastSeq, level: frame.level, message: frame.message },
          ],
        })
        break
      case 'resync':
        set({ revision: frame.revision })
        break
    }
  },

  setConnected: connected => set({ connected }),
  setSession: session => set({ session }),
  setDownloads: downloads => set({ downloads }),
  setPlayer: player => set({ player }),
  setConfig: config => set({ config }),
  setSearchConfigured: configured => set({ searchConfigured: configured }),
  optimisticSelect: (indices, selected) => {
    const session = get().session
    if (!session) return
    set({ session: recomputeSelection(session, indices, selected) })
  },
  optimisticRemoveDownload: id =>
    set(state => ({ downloads: state.downloads.filter(download => download.id !== id) })),
  optimisticRename: (index, name) => {
    const session = get().session
    if (!session) return
    set({
      session: {
        ...session,
        files: session.files.map(file =>
          file.index === index ? { ...file, name } : file,
        ),
      },
    })
  },
  addToast: (level, message) => {
    const id = ++toastSeq
    set(state => ({ toasts: [...state.toasts, { id, level, message }] }))
    return id
  },
  dismissToast: id =>
    set(state => ({ toasts: state.toasts.filter(toast => toast.id !== id) })),
}))
