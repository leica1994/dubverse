export interface AiConfig {
  id: string
  title: string
  baseUrl: string
  apiKey: string
  model: string
  sortOrder: number
  isDefault: boolean
  concurrentLimit: number
  requestTimeout: number
  rateLimit: number
}

export const AI_CONFIG_DEFAULTS: Omit<AiConfig, 'id'> = {
  title: '',
  baseUrl: 'https://api.openai.com/v1',
  apiKey: '',
  model: 'gpt-4o-mini',
  sortOrder: 0,
  isDefault: false,
  concurrentLimit: 5,
  requestTimeout: 180,
  rateLimit: 60,
}
