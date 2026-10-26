import { create } from 'zustand'

export type MotionPreference = 'system' | 'full' | 'reduced'

interface UiState {
  commandOpen: boolean
  shortcutsOpen: boolean
  motion: MotionPreference
  activityOpen: boolean
  setCommandOpen: (open: boolean) => void
  setShortcutsOpen: (open: boolean) => void
  setMotion: (motion: MotionPreference) => void
  setActivityOpen: (open: boolean) => void
}

function readMotion(): MotionPreference {
  if (typeof localStorage === 'undefined') return 'system'
  const saved = localStorage.getItem('twatch.motion')
  return saved === 'full' || saved === 'reduced' || saved === 'system' ? saved : 'system'
}

export const useUiStore = create<UiState>(set => ({
  commandOpen: false,
  shortcutsOpen: false,
  motion: readMotion(),
  activityOpen: false,
  setCommandOpen: commandOpen => set({ commandOpen }),
  setShortcutsOpen: shortcutsOpen => set({ shortcutsOpen }),
  setMotion: motion => {
    localStorage.setItem('twatch.motion', motion)
    set({ motion })
  },
  setActivityOpen: activityOpen => set({ activityOpen }),
}))
