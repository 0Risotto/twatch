;(() => {
  try {
    const theme = localStorage.getItem('twatch.theme')
    if (theme) document.documentElement.dataset.theme = theme
    const mode = localStorage.getItem('twatch.mode')
    if (mode) document.documentElement.dataset.mode = mode
  } catch {
    /* storage is optional */
  }
})()
