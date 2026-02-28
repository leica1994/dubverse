import { ref, watch } from 'vue'
import { setConfig } from './useDatabase'

interface TranslationSettings {
  correction: boolean
  optimization: boolean
  promptType: 'standard' | 'reflective'
  batchSize: number
  worldBuilding: string
  writingStyle: string
  glossary: string
  forbidden: string
  examples: string
  customPrompt: string
}

const defaults: TranslationSettings = {
  correction: false,
  optimization: false,
  promptType: 'standard',
  batchSize: 30,
  worldBuilding: '',
  writingStyle: '',
  glossary: '',
  forbidden: '',
  examples: '',
  customPrompt: '',
}

const translationSettings = ref<TranslationSettings>({ ...defaults })
const isLoaded = ref(false)

const KEY_MAP: Record<keyof TranslationSettings, string> = {
  correction: 'translation.correction',
  optimization: 'translation.optimization',
  promptType: 'translation.prompt_type',
  batchSize: 'translation.ai.batch_size',
  worldBuilding: 'translation.world_building',
  writingStyle: 'translation.writing_style',
  glossary: 'translation.glossary',
  forbidden: 'translation.forbidden',
  examples: 'translation.examples',
  customPrompt: 'translation.custom_prompt',
}

export async function initTranslationSettings(dbConfig: Record<string, string>) {
  translationSettings.value = {
    correction: dbConfig[KEY_MAP.correction] === 'true',
    optimization: dbConfig[KEY_MAP.optimization] === 'true',
    promptType: (dbConfig[KEY_MAP.promptType] as 'standard' | 'reflective') ?? defaults.promptType,
    batchSize: parseInt(dbConfig[KEY_MAP.batchSize] ?? '', 10) || defaults.batchSize,
    worldBuilding: dbConfig[KEY_MAP.worldBuilding] ?? '',
    writingStyle: dbConfig[KEY_MAP.writingStyle] ?? '',
    glossary: dbConfig[KEY_MAP.glossary] ?? '',
    forbidden: dbConfig[KEY_MAP.forbidden] ?? '',
    examples: dbConfig[KEY_MAP.examples] ?? '',
    customPrompt: dbConfig[KEY_MAP.customPrompt] ?? '',
  }
  isLoaded.value = true
}

watch(
  translationSettings,
  async (val) => {
    if (!isLoaded.value) return
    await setConfig(KEY_MAP.correction, String(val.correction))
    await setConfig(KEY_MAP.optimization, String(val.optimization))
    await setConfig(KEY_MAP.promptType, val.promptType)
    await setConfig(KEY_MAP.batchSize, String(val.batchSize))
    await setConfig(KEY_MAP.worldBuilding, val.worldBuilding)
    await setConfig(KEY_MAP.writingStyle, val.writingStyle)
    await setConfig(KEY_MAP.glossary, val.glossary)
    await setConfig(KEY_MAP.forbidden, val.forbidden)
    await setConfig(KEY_MAP.examples, val.examples)
    await setConfig(KEY_MAP.customPrompt, val.customPrompt)
  },
  { deep: true },
)

export function useTranslationSettings() {
  return { translationSettings }
}
