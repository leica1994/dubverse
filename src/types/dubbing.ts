export type ReferenceMode = 'none' | 'custom' | 'clone'
export type DubbingStage = 'preprocess' | 'media' | 'reference' | 'tts' | 'alignment' | 'compose'
export type DubbingStatus = 'pending' | 'running' | 'completed' | 'failed'

export const DUBBING_STAGE_LABELS: Record<DubbingStage, string> = {
  preprocess: '字幕预处理',
  media: '媒体分离',
  reference: '参考音频',
  tts: 'TTS 生成',
  alignment: '音频对齐',
  compose: '视频合成',
}

export interface TtsPlugin {
  id: string
  name: string
  pluginType: 'ncn' | 'gradio' | 'http_rest'
  configJson: string
  requiresRef: boolean
  isEnabled: boolean
  sortOrder: number
  createdAt: string
}

export interface StageState {
  jobId: string
  stage: DubbingStage
  status: DubbingStatus
  progress: number
  outputPath?: string
  error?: string
  completedAt?: string
}

export interface DubbingJobInfo {
  id: string
  projectDir: string
  videoPath: string
  subtitleCount: number
  referenceMode: ReferenceMode
  referenceAudioPath?: string
  ttsPluginId?: string
  status: DubbingStatus
  currentStage?: DubbingStage
  error?: string
  stages: StageState[]
}

export interface TtsItemProgress {
  index: number
  status: 'pending' | 'completed' | 'failed'
  audioPath?: string
  error?: string
}

// Progress events emitted from Rust
export interface DubbingProgressEvent {
  stage: DubbingStage
  percent: number
  message: string
}

export interface DubbingStageChangeEvent {
  stage: DubbingStage
  status: DubbingStatus
}

export interface DubbingTtsItemDoneEvent {
  index: number
  status: 'completed' | 'failed'
  audioPath?: string
}

// TTS plugin config types
export interface NcnConfig {
  voiceId: string
}

export interface GradioConfig {
  endpoint: string
  inputComponents?: string[]
}

export interface HttpRestConfig {
  url: string
  method: 'POST' | 'GET'
  textKey: string
  referenceAudioKey?: string
  voiceKey?: string
  voiceId?: string
  responseType: 'json_base64' | 'binary' | 'file_url'
  responseKey?: string
  headers?: Record<string, string>
}

export const TTS_PLUGIN_TYPE_LABELS: Record<TtsPlugin['pluginType'], string> = {
  ncn: 'N.CN (免费)',
  gradio: 'Gradio (声音克隆)',
  http_rest: '自定义 HTTP REST',
}
