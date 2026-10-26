import type { ThemeMode } from '@/types/api'

/** Built-in palettes. Each base has a dark and light variant. */
export const THEME_BASES = [
  { value: 'tokyonight', label: 'Tokyo Night', emoji: '🌃' },
  { value: 'kanagawa', label: 'Kanagawa', emoji: '🎏' },
  { value: 'nord', label: 'Nord', emoji: '🌊' },
  { value: 'gruvbox', label: 'Gruvbox', emoji: '🍦' },
  { value: 'catppuccin', label: 'Catppuccin', emoji: '🐱' },
  { value: 'purple', label: 'Purple', emoji: '🟣' },
  { value: 'emerald', label: 'Emerald', emoji: '🌿' },
  { value: 'monokai', label: 'Monokai', emoji: '🎨' },
  { value: 'solarized', label: 'Solarized', emoji: '☀️' },
  { value: 'amber', label: 'Amber', emoji: '🔥' },
  { value: 'dracula', label: 'Dracula', emoji: '🔮' },
  { value: 'rosepine', label: 'Rosé Pine', emoji: '🌹' },
  { value: 'ice', label: 'Ice', emoji: '🧊' },
  { value: 'amoled', label: 'AMOLED', emoji: '🖤' },
  { value: 'orchid', label: 'Orchid', emoji: '🪻' },
  { value: 'intellij', label: 'IntelliJ', emoji: '😶‍🌫️' },
  { value: 'highcontrast', label: 'High Contrast', emoji: '🔆' },
  { value: 'everforest', label: 'Everforest', emoji: '🌲' },
  { value: 'amoled-midnight', label: 'AMOLED Midnight', emoji: '🌌' },
  { value: 'amoled-grey-calm', label: 'AMOLED Grey Calm', emoji: '🌑' },
  { value: 'nu-disco', label: 'Nu Disco', emoji: '🪩' },
  { value: 'amethyst-oled', label: 'Amethyst OLED', emoji: '💜' },
  { value: 'ocean', label: 'Ocean', emoji: '🐋' },
  { value: 'forest', label: 'Forest', emoji: '🌳' },
  { value: 'sunset', label: 'Sunset', emoji: '🌅' },
  { value: 'sakura', label: 'Sakura', emoji: '🌸' },
  { value: 'coffee', label: 'Coffee', emoji: '☕' },
] as const

export type ThemeBase = (typeof THEME_BASES)[number]['value']

export function splitTheme(name: string): { base: ThemeBase; mode: ThemeMode } {
  const mode: ThemeMode = name.endsWith('-light') ? 'light' : 'dark'
  const base = name.slice(0, -(mode.length + 1)) as ThemeBase
  return { base, mode }
}

export function themeName(base: string, mode: ThemeMode): string {
  return `${base}-${mode}`
}

export function themeLabel(base: string): string {
  return THEME_BASES.find(theme => theme.value === base)?.label ?? base
}

export function isKnownTheme(name: string): boolean {
  return THEME_BASES.some(theme => name === `${theme.value}-dark` || name === `${theme.value}-light`)
}

export function applyTheme(name: string): void {
  const effective = isKnownTheme(name) ? name : 'emerald-dark'
  const { mode } = splitTheme(effective)
  const root = document.documentElement
  root.dataset.theme = effective
  root.dataset.mode = mode
  try {
    localStorage.setItem('twatch.theme', effective)
    localStorage.setItem('twatch.mode', mode)
  } catch {
    // Storage is optional.
  }
}
