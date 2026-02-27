export type TranscriptionProviderId = 'bcut' | 'elevenlabs-free' | 'elevenlabs-paid'

export type BcutConfig = Record<string, never>

export interface ElevenLabsFreeConfig {
  modelId: 'scribe_v2' | 'scribe_v1'
  numSpeakers: number
  tagAudioEvents: boolean
}

export interface ElevenLabsPaidConfig {
  apiKey: string
  modelId: 'scribe_v2' | 'scribe_v1'
  numSpeakers: number
  enableDiarization: boolean
  tagAudioEvents: boolean
}

export interface ProviderConfigMap {
  'bcut': BcutConfig
  'elevenlabs-free': ElevenLabsFreeConfig
  'elevenlabs-paid': ElevenLabsPaidConfig
}

export type FieldType = 'text' | 'password' | 'select' | 'toggle' | 'number'

export interface ConfigFieldSchema {
  key: string
  label: string
  type: FieldType
  required?: boolean
  min?: number
  max?: number
  options?: { value: string | number | boolean; label: string }[]
  placeholder?: string
  hint?: string
  sensitive?: boolean
}

export interface TranscriptionProvider<Id extends TranscriptionProviderId = TranscriptionProviderId> {
  id: Id
  name: string
  description: string
  requiresApiKey: boolean
  defaultConfig: ProviderConfigMap[Id]
  configSchema: ConfigFieldSchema[]
  validate: (config: ProviderConfigMap[Id]) => { valid: boolean; errors: Record<string, string> }
}

export interface TranscriptionSettings {
  activeProviderId: TranscriptionProviderId
  configs: {
    bcut: BcutConfig
    'elevenlabs-free': ElevenLabsFreeConfig
    'elevenlabs-paid': ElevenLabsPaidConfig
  }
}
