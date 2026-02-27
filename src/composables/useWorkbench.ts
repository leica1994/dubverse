import { ref, computed } from 'vue'
import type {
  StepStatus, VideoFile, Subtitle, TTSConfig, ExportConfig, ProgressInfo,
} from '@/types/workbench'
import { STEP_LABELS } from '@/types/workbench'

const currentStep = ref(0)
const stepStatuses = ref<StepStatus[]>(['ready', 'idle', 'idle', 'idle', 'idle', 'idle'])
const videoFile = ref<VideoFile | null>(null)
const sourceLanguage = ref('zh')
const targetLanguage = ref('en')
const originalSubtitles = ref<Subtitle[]>([])
const translatedSubtitles = ref<Subtitle[]>([])
const ttsConfig = ref<TTSConfig>({ voiceId: '', speed: 1.0, pitch: 1.0 })
const exportConfig = ref<ExportConfig>({ format: 'mp4', quality: 'high', outputPath: '' })
const progress = ref<ProgressInfo>({ phase: '', percent: 0, message: '' })
const projectDir = ref<string>('')

const isProcessing = computed(() => stepStatuses.value.some((s: StepStatus) => s === 'processing'))

const canGoNext = computed(() => {
  const status = stepStatuses.value[currentStep.value]
  return status === 'completed' && currentStep.value < STEP_LABELS.length - 1
})

const canGoPrev = computed(() => currentStep.value > 0 && !isProcessing.value)

function invalidateFrom(stepIndex: number) {
  for (let i = stepIndex; i < stepStatuses.value.length; i++) {
    stepStatuses.value[i] = 'idle'
  }
}

function goToStep(step: number) {
  if (isProcessing.value) return
  if (step < 0 || step >= STEP_LABELS.length) return
  const targetStatus = stepStatuses.value[step]
  if (targetStatus === 'idle') return
  currentStep.value = step
}

function goNext() {
  if (canGoNext.value) {
    currentStep.value++
  }
}

function goPrev() {
  if (canGoPrev.value) {
    currentStep.value--
  }
}

function setStepStatus(step: number, status: StepStatus) {
  stepStatuses.value[step] = status
  if (status === 'completed' && step + 1 < STEP_LABELS.length) {
    if (stepStatuses.value[step + 1] === 'idle') {
      stepStatuses.value[step + 1] = 'ready'
    }
  }
}

function setVideoFile(file: VideoFile) {
  videoFile.value = file
  setStepStatus(0, 'completed')
  invalidateFrom(1)
  stepStatuses.value[1] = 'ready'
}

function clearVideoFile() {
  videoFile.value = null
  setStepStatus(0, 'ready')
  invalidateFrom(1)
}

function resetWorkbench() {
  currentStep.value = 0
  stepStatuses.value = ['ready', 'idle', 'idle', 'idle', 'idle', 'idle']
  videoFile.value = null
  sourceLanguage.value = 'zh'
  targetLanguage.value = 'en'
  originalSubtitles.value = []
  translatedSubtitles.value = []
  ttsConfig.value = { voiceId: '', speed: 1.0, pitch: 1.0 }
  exportConfig.value = { format: 'mp4', quality: 'high', outputPath: '' }
  progress.value = { phase: '', percent: 0, message: '' }
  projectDir.value = ''
}

export function useWorkbench() {
  return {
    currentStep,
    stepStatuses,
    videoFile,
    sourceLanguage,
    targetLanguage,
    originalSubtitles,
    translatedSubtitles,
    ttsConfig,
    exportConfig,
    progress,
    projectDir,
    isProcessing,
    canGoNext,
    canGoPrev,
    goToStep,
    goNext,
    goPrev,
    setStepStatus,
    setVideoFile,
    clearVideoFile,
    invalidateFrom,
    resetWorkbench,
  }
}
