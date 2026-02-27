export type StepStatus = 'idle' | 'ready' | 'processing' | 'completed' | 'error'

export interface VideoFile {
  name: string
  path: string
  size: number
  duration: number
  width: number
  height: number
  thumbnailUrl?: string
}

export interface Subtitle {
  id: number
  startTime: number
  endTime: number
  text: string
}

export interface TTSVoice {
  id: string
  name: string
  gender: 'male' | 'female'
  language: string
  sampleUrl?: string
}

export interface TTSConfig {
  voiceId: string
  speed: number
  pitch: number
}

export interface ExportConfig {
  format: 'mp4' | 'mkv' | 'webm'
  quality: 'high' | 'medium' | 'low'
  outputPath: string
}

export interface LanguagePair {
  source: string
  target: string
}

export interface ProgressInfo {
  phase: string
  percent: number
  message: string
}

export const STEP_LABELS = ['上传', '转录', '翻译', '审阅', '配音', '导出'] as const

export const LANGUAGES = [
  { code: 'zh', label: '中文' },
  { code: 'en', label: 'English' },
  { code: 'ja', label: '日本語' },
  { code: 'ko', label: '한국어' },
  { code: 'es', label: 'Español' },
  { code: 'fr', label: 'Français' },
  { code: 'de', label: 'Deutsch' },
  { code: 'ru', label: 'Русский' },
] as const

export const MOCK_VOICES: TTSVoice[] = [
  { id: 'v1', name: '晓晓', gender: 'female', language: 'zh' },
  { id: 'v2', name: '云扬', gender: 'male', language: 'zh' },
  { id: 'v3', name: 'Jenny', gender: 'female', language: 'en' },
  { id: 'v4', name: 'Guy', gender: 'male', language: 'en' },
  { id: 'v5', name: 'Nanami', gender: 'female', language: 'ja' },
  { id: 'v6', name: 'Keita', gender: 'male', language: 'ja' },
]
