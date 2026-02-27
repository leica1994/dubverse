import { createApp } from "vue"
import App from "./App.vue"
import router from "./router"
import "./styles/variables.css"
import { useSettings } from "./composables/useSettings"

const { applyTheme } = useSettings()
applyTheme()

const app = createApp(App)
app.use(router)
app.mount("#app")

// Close-to-tray intercept
import { getCurrentWindow } from "@tauri-apps/api/window"

getCurrentWindow().onCloseRequested(async (event) => {
  const { settings } = useSettings()
  if (settings.value.closeToTray) {
    event.preventDefault()
    await getCurrentWindow().hide()
  }
})
