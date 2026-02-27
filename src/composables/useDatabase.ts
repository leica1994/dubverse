import { invoke } from '@tauri-apps/api/core'

export async function getAllConfig(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>('cmd_get_all_config')
}

export async function setConfig(key: string, value: string): Promise<void> {
  return invoke('cmd_set_config', { key, value })
}

export async function getProviderSecret(providerId: string): Promise<string | null> {
  return invoke<string | null>('cmd_get_provider_secret', { providerId })
}

export async function setProviderSecret(providerId: string, secretJson: string): Promise<void> {
  return invoke('cmd_set_provider_secret', { providerId, secretJson })
}
