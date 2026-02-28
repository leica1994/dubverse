import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type {
  DubbingJobInfo, DubbingStage, DubbingStatus,
  DubbingProgressEvent, DubbingStageChangeEvent, DubbingTtsItemDoneEvent,
  TtsItemProgress, StageState, ReferenceMode,
} from '@/types/dubbing'
import type { Subtitle } from '@/types/workbench'

// ── State ─────────────────────────────────────────────────────────────────────

const jobInfo = ref<DubbingJobInfo | null>(null)
const stageProgress = ref<Record<DubbingStage, number>>({
  preprocess: 0, media: 0, reference: 0, tts: 0, alignment: 0, compose: 0,
})
const stageStatuses = ref<Record<DubbingStage, DubbingStatus>>({
  preprocess: 'pending', media: 'pending', reference: 'pending',
  tts: 'pending', alignment: 'pending', compose: 'pending',
})
const ttsItemProgress = ref<Map<number, TtsItemProgress>>(new Map())
const currentMessage = ref('')
const isRunning = ref(false)
const outputPath = ref('')
const preprocessedTexts = ref<string[]>([])
const silentVideoPath = ref('')
const vocalAudioPath = ref('')

const hasResumableJob = computed(() => {
  if (!jobInfo.value) return false
  return jobInfo.value.status === 'running' || (
    jobInfo.value.status === 'pending' &&
    jobInfo.value.stages.some(s => s.status === 'completed')
  )
})

const overallPercent = computed(() => {
  const stages: DubbingStage[] = ['preprocess', 'media', 'reference', 'tts', 'alignment', 'compose']
  const total = stages.reduce((sum, s) => sum + stageProgress.value[s], 0)
  return total / stages.length
})

// ── Event Listeners ──────────────────────────────────────────────────────────

let unlistenProgress: UnlistenFn | null = null
let unlistenStageChange: UnlistenFn | null = null
let unlistenTtsItem: UnlistenFn | null = null

async function startListening() {
  unlistenProgress = await listen<DubbingProgressEvent>('dubbing:progress', ({ payload }) => {
    stageProgress.value[payload.stage] = payload.percent
    currentMessage.value = payload.message
  })
  unlistenStageChange = await listen<DubbingStageChangeEvent>('dubbing:stage_change', ({ payload }) => {
    stageStatuses.value[payload.stage] = payload.status as DubbingStatus
  })
  unlistenTtsItem = await listen<DubbingTtsItemDoneEvent>('dubbing:tts_item_done', ({ payload }) => {
    ttsItemProgress.value.set(payload.index, {
      index: payload.index,
      status: payload.status,
      audioPath: payload.audioPath,
    })
  })
}

function stopListening() {
  unlistenProgress?.()
  unlistenStageChange?.()
  unlistenTtsItem?.()
  unlistenProgress = null
  unlistenStageChange = null
  unlistenTtsItem = null
}

// ── Actions ──────────────────────────────────────────────────────────────────

async function checkResumableJob(projectDir: string): Promise<boolean> {
  try {
    const raw = await invoke<Record<string, unknown> | null>('cmd_get_dubbing_job', { projectDir })
    if (!raw) return false
    jobInfo.value = rawToJobInfo(raw)
    // Restore stage statuses from DB
    for (const stage of jobInfo.value.stages) {
      stageStatuses.value[stage.stage as DubbingStage] = stage.status
      stageProgress.value[stage.stage as DubbingStage] = stage.status === 'completed' ? 100 : 0
    }
    return jobInfo.value.status !== 'completed'
  } catch {
    return false
  }
}

async function initJob(params: {
  projectDir: string
  videoPath: string
  subtitleCount: number
  referenceMode: ReferenceMode
  referenceAudioPath?: string
  ttsPluginId?: string
}): Promise<string> {
  const raw = await invoke<Record<string, unknown>>('cmd_init_dubbing_job', {
    projectDir: params.projectDir,
    videoPath: params.videoPath,
    subtitleCount: params.subtitleCount,
    referenceMode: params.referenceMode,
    referenceAudioPath: params.referenceAudioPath ?? null,
    ttsPluginId: params.ttsPluginId ?? null,
  })
  jobInfo.value = rawToJobInfo(raw)
  return jobInfo.value.id
}

async function resetJob(): Promise<void> {
  if (!jobInfo.value) return
  await invoke('cmd_reset_dubbing_job', { jobId: jobInfo.value.id })
  jobInfo.value = null
  resetState()
}

function resetState() {
  Object.keys(stageProgress.value).forEach(k => {
    stageProgress.value[k as DubbingStage] = 0
  })
  Object.keys(stageStatuses.value).forEach(k => {
    stageStatuses.value[k as DubbingStage] = 'pending'
  })
  ttsItemProgress.value.clear()
  currentMessage.value = ''
  isRunning.value = false
  outputPath.value = ''
  preprocessedTexts.value = []
  silentVideoPath.value = ''
  vocalAudioPath.value = ''
}

async function cancel(): Promise<void> {
  await invoke('cmd_cancel_dubbing')
  isRunning.value = false
}

async function runPreprocess(
  jobId: string,
  subtitles: Subtitle[],
  batchSize?: number,
): Promise<string[]> {
  const entries = subtitles.map(s => ({
    id: s.id,
    startTime: s.startTime,
    endTime: s.endTime,
    text: s.text,
  }))
  return invoke<string[]>('cmd_run_preprocess', {
    jobId,
    subtitles: entries,
    batchSize: batchSize ?? null,
  })
}

async function runMediaSeparation(jobId: string, videoPath: string, workDir: string) {
  return invoke<{ vocalAudioPath: string; silentVideoPath: string }>(
    'cmd_run_media_separation',
    { jobId, videoPath, workDir },
  )
}

async function runReferenceGeneration(params: {
  jobId: string
  referenceMode: ReferenceMode
  vocalAudioPath?: string
  customAudioPath?: string
  subtitleEntries: Subtitle[]
  workDir: string
}) {
  return invoke('cmd_run_reference_generation', {
    jobId: params.jobId,
    referenceMode: params.referenceMode,
    vocalAudioPath: params.vocalAudioPath ?? null,
    customAudioPath: params.customAudioPath ?? null,
    subtitleEntries: params.subtitleEntries.map(s => ({
      id: s.id, startTime: s.startTime, endTime: s.endTime, text: s.text,
    })),
    workDir: params.workDir,
  })
}

async function initTtsItems(jobId: string, subtitles: Subtitle[], preprocessed: string[]) {
  const items = subtitles.map((s, i) => ({
    id: s.id,
    startTime: s.startTime,
    endTime: s.endTime,
    preprocessedText: preprocessed[i] || s.text,
  }))
  return invoke('cmd_init_tts_items', { jobId, subtitles: items })
}

async function runTtsGeneration(jobId: string, workDir: string, options: { pluginId?: string; ncnVoiceId?: string }) {
  return invoke<{ completed: number; total: number }>(
    'cmd_run_tts_generation',
    { jobId, pluginId: options.pluginId ?? null, ncnVoiceId: options.ncnVoiceId ?? null, workDir },
  )
}

async function runAlignmentAndCompose(params: {
  jobId: string
  silentVideoPath: string
  workDir: string
  outputPath: string
}) {
  return invoke<{ outputPath: string }>('cmd_run_alignment_and_compose', {
    jobId: params.jobId,
    silentVideoPath: params.silentVideoPath,
    workDir: params.workDir,
    outputPath: params.outputPath,
  })
}

// ── Helpers ──────────────────────────────────────────────────────────────────

function rawToJobInfo(raw: Record<string, unknown>): DubbingJobInfo {
  const rawStages = (raw.stages as Record<string, unknown>[] | undefined) ?? []
  return {
    id: raw.id as string,
    projectDir: raw.projectDir as string,
    videoPath: raw.videoPath as string,
    subtitleCount: raw.subtitleCount as number,
    referenceMode: raw.referenceMode as ReferenceMode,
    referenceAudioPath: raw.referenceAudioPath as string | undefined,
    ttsPluginId: raw.ttsPluginId as string | undefined,
    status: raw.status as DubbingStatus,
    currentStage: raw.currentStage as DubbingStage | undefined,
    error: raw.error as string | undefined,
    stages: rawStages.map(s => ({
      jobId: s.jobId as string,
      stage: s.stage as DubbingStage,
      status: s.status as DubbingStatus,
      progress: s.progress as number,
      outputPath: s.outputPath as string | undefined,
      error: s.error as string | undefined,
      completedAt: s.completedAt as string | undefined,
    })) as StageState[],
  }
}

export function useDubbing() {
  return {
    jobInfo,
    stageProgress,
    stageStatuses,
    ttsItemProgress,
    currentMessage,
    isRunning,
    outputPath,
    preprocessedTexts,
    silentVideoPath,
    vocalAudioPath,
    hasResumableJob,
    overallPercent,
    // actions
    startListening,
    stopListening,
    checkResumableJob,
    initJob,
    resetJob,
    resetState,
    cancel,
    runPreprocess,
    runMediaSeparation,
    runReferenceGeneration,
    initTtsItems,
    runTtsGeneration,
    runAlignmentAndCompose,
  }
}
