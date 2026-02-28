import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type {
  StepStatus, VideoFile, Subtitle, TTSConfig, ExportConfig, ProgressInfo, WorkbenchTaskFull,
} from '@/types/workbench'
import { STEP_LABELS } from '@/types/workbench'

const currentStep = ref(0)
const stepStatuses = ref<StepStatus[]>(['ready', 'idle', 'idle', 'idle', 'idle'])
const videoFile = ref<VideoFile | null>(null)
const sourceLanguage = ref('auto')
const targetLanguage = ref('zh')
const originalSubtitles = ref<Subtitle[]>([])
const translatedSubtitles = ref<Subtitle[]>([])
const ttsConfig = ref<TTSConfig>({ voiceId: '', speed: 1.0, pitch: 1.0 })
const exportConfig = ref<ExportConfig>({ format: 'mp4', quality: 'high', outputPath: '' })
const progress = ref<ProgressInfo>({ phase: '', percent: 0, message: '' })
const projectDir = ref<string>('')

const isProcessing = computed(() => stepStatuses.value.some((s: StepStatus) => s === 'processing'))

const workbenchTaskId = ref<string>('')

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
  saveProgress()
}

function goNext() {
  if (canGoNext.value) {
    currentStep.value++
    saveProgress()
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
  stepStatuses.value = ['ready', 'idle', 'idle', 'idle', 'idle']
  videoFile.value = null
  sourceLanguage.value = 'auto'
  targetLanguage.value = 'zh'
  originalSubtitles.value = []
  translatedSubtitles.value = []
  ttsConfig.value = { voiceId: '', speed: 1.0, pitch: 1.0 }
  exportConfig.value = { format: 'mp4', quality: 'high', outputPath: '' }
  progress.value = { phase: '', percent: 0, message: '' }
  projectDir.value = ''
  workbenchTaskId.value = ''
}

async function createTask(): Promise<void> {
  if (!videoFile.value || !projectDir.value || workbenchTaskId.value) return
  try {
    const result = await invoke<{ id: string }>('cmd_create_workbench_task', {
      videoPath: videoFile.value.path,
      videoName: videoFile.value.name,
      videoSize: videoFile.value.size,
      videoDuration: videoFile.value.duration,
      videoWidth: videoFile.value.width,
      videoHeight: videoFile.value.height,
      projectDir: projectDir.value,
      sourceLanguage: sourceLanguage.value,
      targetLanguage: targetLanguage.value,
    })
    workbenchTaskId.value = result.id
  } catch {
    // 静默失败，不影响主流程
  }
}

async function saveProgress(): Promise<void> {
  if (!workbenchTaskId.value) return
  try {
    // Replace any lingering 'processing' state with 'ready' before saving
    const statuses = stepStatuses.value.map((s) =>
      s === 'processing' ? 'ready' : s
    ) as StepStatus[]
    await invoke('cmd_update_workbench_task_progress', {
      taskId: workbenchTaskId.value,
      currentStep: currentStep.value,
      stepStatuses: statuses,
      sourceLanguage: sourceLanguage.value,
      targetLanguage: targetLanguage.value,
      status: stepStatuses.value[3] === 'completed' ? 'completed' : 'active',
    })
  } catch {
    // 静默失败，不影响主流程
  }
}

async function restoreTask(task: WorkbenchTaskFull): Promise<void> {
  workbenchTaskId.value = task.id
  projectDir.value = task.projectDir
  videoFile.value = {
    name: task.videoName,
    path: task.videoPath,
    size: task.videoSize,
    duration: task.videoDuration,
    width: task.videoWidth,
    height: task.videoHeight,
  }
  sourceLanguage.value = task.sourceLanguage
  targetLanguage.value = task.targetLanguage
  currentStep.value = task.currentStep

  const parsed = JSON.parse(task.stepStatuses) as StepStatus[]
  // Reset any 'processing' state to 'ready' (app may have been killed mid-task)
  stepStatuses.value = parsed.map((s) => (s === 'processing' ? 'ready' : s)) as StepStatus[]

  if (task.stepTranscribe?.subtitlesPath) {
    const json = await invoke<string>('cmd_load_subtitles', {
      subtitlesPath: task.stepTranscribe.subtitlesPath,
    }).catch(() => '[]')
    originalSubtitles.value = JSON.parse(json)
  }
  if (task.stepTranslate?.translatedSubtitlesPath) {
    const json = await invoke<string>('cmd_load_subtitles', {
      subtitlesPath: task.stepTranslate.translatedSubtitlesPath,
    }).catch(() => '[]')
    translatedSubtitles.value = JSON.parse(json)
  }
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
    workbenchTaskId,
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
    createTask,
    saveProgress,
    restoreTask,
  }
}
