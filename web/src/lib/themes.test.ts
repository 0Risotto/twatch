import { describe, expect, it } from 'vitest'
import { applyTheme, splitTheme, themeLabel, themeName } from './themes'

describe('theme helpers', () => {
  it('splits theme ids into base and mode', () => {
    expect(splitTheme('emerald-dark')).toEqual({ base: 'emerald', mode: 'dark' })
    expect(splitTheme('tokyonight-light')).toEqual({ base: 'tokyonight', mode: 'light' })
    expect(splitTheme('amoled-midnight-light')).toEqual({ base: 'amoled-midnight', mode: 'light' })
  })

  it('builds ids and labels', () => {
    expect(themeName('rosepine', 'light')).toBe('rosepine-light')
    expect(themeLabel('catppuccin')).toBe('Catppuccin')
    expect(themeLabel('missing')).toBe('missing')
  })

  it('applies data attributes and persists the choice', () => {
    applyTheme('orchid-dark')
    expect(document.documentElement.dataset.theme).toBe('orchid-dark')
    expect(document.documentElement.dataset.mode).toBe('dark')
    expect(localStorage.getItem('twatch.theme')).toBe('orchid-dark')
  })
})
