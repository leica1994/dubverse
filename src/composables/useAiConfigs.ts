import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AiConfig } from '../types/ai-config'

const aiConfigs = ref<AiConfig[]>([])

const defaultConfig = computed(() => aiConfigs.value.find(c => c.isDefault))

// ── camelCase ↔ snake_case mapping ──────────────────────────────────────────

function toFrontend(raw: Record<string, unknown>): AiConfig {
  return {
    id: raw.id as string,
    title: raw.title as string,
    baseUrl: raw.base_url as string,
    apiKey: raw.api_key as string,
    model: raw.model as string,
    sortOrder: raw.sort_order as number,
    isDefault: raw.is_default === 1 || raw.is_default === true,
    concurrentLimit: raw.concurrent_limit as number,
    requestTimeout: raw.request_timeout as number,
    rateLimit: raw.rate_limit as number,
  }
}

function toBackend(c: AiConfig): Record<string, unknown> {
  return {
    id: c.id,
    title: c.title,
    base_url: c.baseUrl,
    api_key: c.apiKey,
    model: c.model,
    sort_order: c.sortOrder,
    is_default: c.isDefault,
    concurrent_limit: c.concurrentLimit,
    request_timeout: c.requestTimeout,
    rate_limit: c.rateLimit,
  }
}

// ── Actions ──────────────────────────────────────────────────────────────────

async function loadAiConfigs(): Promise<void> {
  try {
    const raw = await invoke<Record<string, unknown>[]>('cmd_get_ai_configs')
    aiConfigs.value = raw.map(toFrontend)
  } catch (err) {
    console.error('[useAiConfigs] loadAiConfigs failed', err)
  }
}

async function createAiConfig(config: AiConfig): Promise<string> {
  const id = await invoke<string>('cmd_create_ai_config', { config: toBackend(config) })
  await loadAiConfigs()
  return id
}

async function updateAiConfig(config: AiConfig): Promise<void> {
  await invoke('cmd_update_ai_config', { config: toBackend(config) })
  await loadAiConfigs()
}

async function deleteAiConfig(id: string): Promise<void> {
  await invoke('cmd_delete_ai_config', { id })
  await loadAiConfigs()
}

async function setDefaultAiConfig(id: string): Promise<void> {
  await invoke('cmd_set_default_ai_config', { id })
  await loadAiConfigs()
}

async function testAiConnection(
  baseUrl: string,
  apiKey: string,
  model: string,
): Promise<string> {
  return invoke<string>('cmd_test_ai_connection', {
    baseUrl,
    apiKey,
    model,
  })
}

export function useAiConfigs() {
  return {
    aiConfigs,
    defaultConfig,
    loadAiConfigs,
    createAiConfig,
    updateAiConfig,
    deleteAiConfig,
    setDefaultAiConfig,
    testAiConnection,
  }
}
