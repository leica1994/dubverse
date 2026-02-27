import type {
  TranscriptionProvider,
  BcutConfig,
  ElevenLabsFreeConfig,
  ElevenLabsPaidConfig,
  ConfigFieldSchema,
} from '@/types/transcription'

const MODEL_OPTIONS = [
  { value: 'scribe_v2', label: 'Scribe v2（推荐）' },
  { value: 'scribe_v1', label: 'Scribe v1（旧版本）' },
]

const bcutProvider: TranscriptionProvider<'bcut'> = {
  id: 'bcut',
  name: 'B站转录',
  description: '使用哔哩哔哩字幕识别服务，无需 API 密钥，适合中文内容',
  requiresApiKey: false,
  defaultConfig: {} as BcutConfig,
  configSchema: [] as ConfigFieldSchema[],
  validate: (_config: BcutConfig) => ({ valid: true, errors: {} }),
}

const elevenLabsFreeProvider: TranscriptionProvider<'elevenlabs-free'> = {
  id: 'elevenlabs-free',
  name: 'ElevenLabs 免费版',
  description: '无需账号，使用 ElevenLabs 公开接口，支持多语言转录',
  requiresApiKey: false,
  defaultConfig: {
    modelId: 'scribe_v2',
    numSpeakers: 0,
    tagAudioEvents: false,
  },
  configSchema: [
    {
      key: 'modelId',
      label: '模型版本',
      type: 'select',
      options: MODEL_OPTIONS,
    },
    {
      key: 'numSpeakers',
      label: '说话人数量',
      type: 'number',
      min: 0,
      max: 10,
      hint: '0 表示自动检测，最多支持 10 人',
    },
    {
      key: 'tagAudioEvents',
      label: '标注音频事件',
      type: 'toggle',
      hint: '识别并标注笑声、掌声等非语音事件',
    },
  ] as ConfigFieldSchema[],
  validate: (config: ElevenLabsFreeConfig) => {
    const errors: Record<string, string> = {}
    if (config.numSpeakers < 0 || config.numSpeakers > 10) {
      errors.numSpeakers = '说话人数量须在 0-10 之间'
    }
    return { valid: Object.keys(errors).length === 0, errors }
  },
}

const elevenLabsPaidProvider: TranscriptionProvider<'elevenlabs-paid'> = {
  id: 'elevenlabs-paid',
  name: 'ElevenLabs 付费版',
  description: '使用个人 API 密钥，支持更多说话人和高级功能',
  requiresApiKey: true,
  defaultConfig: {
    apiKey: '',
    modelId: 'scribe_v2',
    numSpeakers: 0,
    enableDiarization: false,
    tagAudioEvents: false,
  },
  configSchema: [
    {
      key: 'apiKey',
      label: 'API 密钥',
      type: 'password',
      required: true,
      sensitive: true,
      placeholder: '输入你的 ElevenLabs API Key',
      hint: '在 ElevenLabs 控制台获取 API 密钥',
    },
    {
      key: 'modelId',
      label: '模型版本',
      type: 'select',
      options: MODEL_OPTIONS,
    },
    {
      key: 'numSpeakers',
      label: '说话人数量',
      type: 'number',
      min: 0,
      max: 32,
      hint: '0 表示自动检测，最多支持 32 人',
    },
    {
      key: 'enableDiarization',
      label: '说话人分离',
      type: 'toggle',
      hint: '区分不同说话人，在字幕中标注说话人标签',
    },
    {
      key: 'tagAudioEvents',
      label: '标注音频事件',
      type: 'toggle',
      hint: '识别并标注笑声、掌声等非语音事件',
    },
  ] as ConfigFieldSchema[],
  validate: (config: ElevenLabsPaidConfig) => {
    const errors: Record<string, string> = {}
    if (!config.apiKey || config.apiKey.trim() === '') {
      errors.apiKey = 'API 密钥不能为空'
    }
    if (config.numSpeakers < 0 || config.numSpeakers > 32) {
      errors.numSpeakers = '说话人数量须在 0-32 之间'
    }
    return { valid: Object.keys(errors).length === 0, errors }
  },
}

export const ALL_PROVIDERS: TranscriptionProvider[] = [
  bcutProvider as TranscriptionProvider,
  elevenLabsFreeProvider as TranscriptionProvider,
  elevenLabsPaidProvider as TranscriptionProvider,
]

export function getProvider(id: string): TranscriptionProvider | undefined {
  return ALL_PROVIDERS.find(p => p.id === id)
}

export { bcutProvider, elevenLabsFreeProvider, elevenLabsPaidProvider }
