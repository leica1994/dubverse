import { ref, computed, watch } from 'vue'
import type { TranscriptionSettings, TranscriptionProviderId } from '@/types/transcription'
import { ALL_PROVIDERS, getProvider } from '@/config/transcription-providers'
import { setConfig, getProviderSecret, setProviderSecret } from './useDatabase'

const defaults: TranscriptionSettings = {
  activeProviderId: 'bcut',
  configs: {
    bcut: {} as Record<string, never>,
    'elevenlabs-free': {
      modelId: 'scribe_v2',
      numSpeakers: 0,
      tagAudioEvents: false,
    },
    'elevenlabs-paid': {
      apiKey: '',
      modelId: 'scribe_v2',
      numSpeakers: 0,
      enableDiarization: true,
      tagAudioEvents: false,
    },
  },
}

const transcriptionSettings = ref<TranscriptionSettings>({
  ...defaults,
  configs: {
    bcut: { ...defaults.configs.bcut },
    'elevenlabs-free': { ...defaults.configs['elevenlabs-free'] },
    'elevenlabs-paid': { ...defaults.configs['elevenlabs-paid'] },
  },
})
const isLoaded = ref(false)

export async function initTranscriptionSettings(dbConfig: Record<string, string>) {
  let bcutConfig = { ...defaults.configs.bcut }
  let elevenFreeConfig = { ...defaults.configs['elevenlabs-free'] }
  let elevenPaidConfig = { ...defaults.configs['elevenlabs-paid'] }
  let apiKey = ''

  try {
    if (dbConfig['transcription.config.bcut']) {
      bcutConfig = { ...bcutConfig, ...JSON.parse(dbConfig['transcription.config.bcut']) }
    }
    if (dbConfig['transcription.config.elevenlabs-free']) {
      elevenFreeConfig = { ...elevenFreeConfig, ...JSON.parse(dbConfig['transcription.config.elevenlabs-free']) }
    }
    if (dbConfig['transcription.config.elevenlabs-paid']) {
      elevenPaidConfig = { ...elevenPaidConfig, ...JSON.parse(dbConfig['transcription.config.elevenlabs-paid']) }
    }
  } catch { /* ignore parse errors, use defaults */ }

  try {
    const secret = await getProviderSecret('elevenlabs-paid')
    if (secret) {
      apiKey = (JSON.parse(secret) as { apiKey?: string }).apiKey ?? ''
    }
  } catch { /* ignore */ }

  transcriptionSettings.value = {
    activeProviderId: (dbConfig['transcription.active_provider'] as TranscriptionProviderId) ?? defaults.activeProviderId,
    configs: {
      bcut: bcutConfig,
      'elevenlabs-free': elevenFreeConfig,
      'elevenlabs-paid': { ...elevenPaidConfig, apiKey },
    },
  }
  isLoaded.value = true
}

watch(
  transcriptionSettings,
  async (val) => {
    if (!isLoaded.value) return
    const { apiKey, ...paidWithoutKey } = val.configs['elevenlabs-paid']
    await setConfig('transcription.active_provider', val.activeProviderId)
    await setConfig('transcription.config.bcut', JSON.stringify(val.configs.bcut))
    await setConfig('transcription.config.elevenlabs-free', JSON.stringify(val.configs['elevenlabs-free']))
    await setConfig('transcription.config.elevenlabs-paid', JSON.stringify(paidWithoutKey))
    await setProviderSecret('elevenlabs-paid', JSON.stringify({ apiKey }))
  },
  { deep: true },
)

function setActiveProvider(id: TranscriptionProviderId) {
  transcriptionSettings.value.activeProviderId = id
}

function updateActiveConfig(key: string, value: unknown) {
  const id = transcriptionSettings.value.activeProviderId
  ;(transcriptionSettings.value.configs[id] as Record<string, unknown>)[key] = value
}

function validateActive() {
  const provider = getProvider(transcriptionSettings.value.activeProviderId)
  if (!provider) return { valid: false, errors: { _: '无效的 Provider' } }
  const config = transcriptionSettings.value.configs[transcriptionSettings.value.activeProviderId]
  return provider.validate(config as never)
}

function resetConfig(id: TranscriptionProviderId) {
  const provider = getProvider(id)
  if (!provider) return
  transcriptionSettings.value.configs[id] = { ...provider.defaultConfig } as never
}

export function useTranscriptionSettings() {
  const activeProvider = computed(() => getProvider(transcriptionSettings.value.activeProviderId)!)
  const activeConfig = computed(() => transcriptionSettings.value.configs[transcriptionSettings.value.activeProviderId])

  return {
    transcriptionSettings,
    activeProvider,
    activeConfig,
    allProviders: ALL_PROVIDERS,
    setActiveProvider,
    updateActiveConfig,
    validateActive,
    resetConfig,
  }
}
