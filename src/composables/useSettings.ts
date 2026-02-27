import { ref, watch } from 'vue'

type Theme = 'dark' | 'light' | 'system'

interface AppSettings {
  theme: Theme
  closeToTray: boolean
}

const STORAGE_KEY = 'dubverse-settings'

const defaults: AppSettings = {
  theme: 'dark',
  closeToTray: false,
}

function loadSettings(): AppSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return { ...defaults, ...JSON.parse(raw) }
  } catch { /* ignore */ }
  return { ...defaults }
}

function saveSettings(s: AppSettings) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(s))
}

const settings = ref<AppSettings>(loadSettings())
let mediaQuery: MediaQueryList | null = null
let mediaHandler: ((e: MediaQueryListEvent) => void) | null = null

function resolveTheme(theme: Theme): 'dark' | 'light' {
  if (theme === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }
  return theme
}

function applyToDOM(resolved: 'dark' | 'light') {
  document.documentElement.setAttribute('data-theme', resolved)
}

function cleanupMediaListener() {
  if (mediaQuery && mediaHandler) {
    mediaQuery.removeEventListener('change', mediaHandler)
    mediaQuery = null
    mediaHandler = null
  }
}

function applyTheme() {
  cleanupMediaListener()
  const resolved = resolveTheme(settings.value.theme)
  applyToDOM(resolved)

  if (settings.value.theme === 'system') {
    mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    mediaHandler = (e: MediaQueryListEvent) => {
      applyToDOM(e.matches ? 'dark' : 'light')
    }
    mediaQuery.addEventListener('change', mediaHandler)
  }
}

function setTheme(theme: Theme) {
  settings.value.theme = theme
  applyTheme()
}

function setCloseToTray(value: boolean) {
  settings.value.closeToTray = value
}

watch(settings, (val) => saveSettings(val), { deep: true })

export function useSettings() {
  return {
    settings,
    setTheme,
    setCloseToTray,
    applyTheme,
  }
}
