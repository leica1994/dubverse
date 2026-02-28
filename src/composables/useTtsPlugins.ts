import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { TtsPlugin } from '@/types/dubbing'

const ttsPlugins = ref<TtsPlugin[]>([])

function toFrontend(raw: Record<string, unknown>): TtsPlugin {
  return {
    id: raw.id as string,
    name: raw.name as string,
    pluginType: raw.pluginType as TtsPlugin['pluginType'],
    configJson: raw.configJson as string,
    requiresRef: raw.requiresRef as boolean,
    isEnabled: raw.isEnabled as boolean,
    sortOrder: raw.sortOrder as number,
    createdAt: raw.createdAt as string,
  }
}

function toBackend(p: TtsPlugin): Record<string, unknown> {
  return {
    id: p.id,
    name: p.name,
    pluginType: p.pluginType,
    configJson: p.configJson,
    requiresRef: p.requiresRef,
    isEnabled: p.isEnabled,
    sortOrder: p.sortOrder,
    createdAt: p.createdAt,
  }
}

async function loadTtsPlugins(): Promise<void> {
  try {
    const raw = await invoke<Record<string, unknown>[]>('cmd_get_tts_plugins')
    ttsPlugins.value = raw.map(toFrontend)
  } catch (err) {
    console.error('[useTtsPlugins] load failed', err)
  }
}

async function createTtsPlugin(plugin: TtsPlugin): Promise<void> {
  await invoke('cmd_create_tts_plugin', { plugin: toBackend(plugin) })
  await loadTtsPlugins()
}

async function updateTtsPlugin(plugin: TtsPlugin): Promise<void> {
  await invoke('cmd_update_tts_plugin', { plugin: toBackend(plugin) })
  await loadTtsPlugins()
}

async function deleteTtsPlugin(id: string): Promise<void> {
  await invoke('cmd_delete_tts_plugin', { id })
  await loadTtsPlugins()
}

async function testTtsPlugin(pluginId: string, sampleText = '你好，这是一段测试文本。'): Promise<string> {
  return invoke<string>('cmd_test_tts_plugin', { pluginId, sampleText })
}

async function listTtsVoices(pluginId: string) {
  return invoke<{ id: string; name: string; description?: string }[]>('cmd_list_tts_voices', { pluginId })
}

export function useTtsPlugins() {
  return {
    ttsPlugins,
    loadTtsPlugins,
    createTtsPlugin,
    updateTtsPlugin,
    deleteTtsPlugin,
    testTtsPlugin,
    listTtsVoices,
  }
}
