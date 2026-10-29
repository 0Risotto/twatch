export const isMacPlatform =
  typeof navigator !== 'undefined' && /Mac|iPhone|iPad|iPod/.test(navigator.platform)

export const commandKeyLabel = isMacPlatform ? '⌘K' : 'Ctrl K'
