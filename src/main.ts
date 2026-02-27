import { createApp } from "vue"
import App from "./App.vue"
import router from "./router"
import "./styles/variables.css"
import { useSettings, initSettings } from "./composables/useSettings"
import { initTranscriptionSettings } from "./composables/useTranscriptionSettings"
import { getAllConfig } from "./composables/useDatabase"
import { getCurrentWindow } from "@tauri-apps/api/window"

async function bootstrap() {
  // 1. Apply default theme immediately to prevent flash
  const { applyTheme, settings } = useSettings()
  applyTheme()

  // 2. Mount Vue with defaults first (renders immediately)
  const app = createApp(App)
  app.use(router)
  app.mount("#app")

  // 3. Fetch all config from DB in one IPC call, then hydrate composables
  try {
    const dbConfig = await getAllConfig()
    await initSettings(dbConfig)
    await initTranscriptionSettings(dbConfig)
    applyTheme() // Re-apply with real theme value
  } catch (err) {
    console.error("[bootstrap] DB load failed, using defaults", err)
  }

  // 4. Bind close-to-tray window event
  getCurrentWindow().onCloseRequested(async (event) => {
    if (settings.value.closeToTray) {
      event.preventDefault()
      await getCurrentWindow().hide()
    }
  })
}

bootstrap()
