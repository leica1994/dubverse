import { ref, watch } from 'vue'
import { setConfig } from './useDatabase'

type Theme = 'dark' | 'light' | 'system'

interface AppSettings {
  theme: Theme
  closeToTray: boolean
  sidebarCollapsed: boolean
}

const defaults: AppSettings = {
  theme: 'dark',
  closeToTray: false,
  sidebarCollapsed: false,
}

const settings = ref<AppSettings>({ ...defaults })
const isLoaded = ref(false)

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

function setSidebarCollapsed(value: boolean) {
  settings.value.sidebarCollapsed = value
}

export async function initSettings(dbConfig: Record<string, string>) {
  settings.value = {
    theme: (dbConfig['ui.theme'] as Theme) ?? defaults.theme,
    closeToTray: dbConfig['system.close_to_tray'] === 'true',
    sidebarCollapsed: dbConfig['ui.sidebar_collapsed'] === 'true',
  }
  isLoaded.value = true
}

watch(
  settings,
  async (val) => {
    if (!isLoaded.value) return
    await setConfig('ui.theme', val.theme)
    await setConfig('system.close_to_tray', String(val.closeToTray))
    await setConfig('ui.sidebar_collapsed', String(val.sidebarCollapsed))
  },
  { deep: true },
)

export function useSettings() {
  return {
    settings,
    setTheme,
    setCloseToTray,
    setSidebarCollapsed,
    applyTheme,
  }
}
