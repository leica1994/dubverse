<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useWorkbench } from '@/composables/useWorkbench'
import { useDubbing } from '@/composables/useDubbing'
import type { ReferenceMode } from '@/types/dubbing'
import ReferenceAudioPicker from '@/components/dubbing/ReferenceAudioPicker.vue'
import TtsPluginSelector from '@/components/dubbing/TtsPluginSelector.vue'
import DubbingProgress from '@/components/dubbing/DubbingProgress.vue'

const {
  projectDir, videoFile, translatedSubtitles, setStepStatus,
} = useWorkbench()

const dubbing = useDubbing()

// Config state
const referenceMode = ref<ReferenceMode>('none')
const customAudioPath = ref('')
const ncnVoiceId = ref<string | undefined>()
const selectedPluginId = ref<string | undefined>()
const resumePrompt = ref(false)

// UI state
const isConfigPhase = computed(() =>
  !dubbing.isRunning.value && !dubbing.jobInfo.value?.status.match(/running|completed/)
)
const isRunning = computed(() => dubbing.isRunning.value)
const isCompleted = computed(() => dubbing.jobInfo.value?.status === 'completed')

const ttsCompletedCount = computed(() => {
  let n = 0
  dubbing.ttsItemProgress.value.forEach(v => { if (v.status === 'completed') n++ })
  return n
})

onMounted(async () => {
  await dubbing.startListening()
  // Check for resumable job
  if (projectDir.value) {
    const resumable = await dubbing.checkResumableJob(projectDir.value)
    if (resumable && dubbing.jobInfo.value) {
      resumePrompt.value = true
      // Restore config from existing job
      const job = dubbing.jobInfo.value
      referenceMode.value = job.referenceMode
      customAudioPath.value = job.referenceAudioPath || ''
      selectedPluginId.value = job.ttsPluginId
      // ncnVoiceId is not persisted; user re-selects on resume
    }
  }
})

onUnmounted(() => {
  dubbing.stopListening()
})

async function startDubbing() {
  if (!projectDir.value || !videoFile.value) return

  dubbing.isRunning.value = true
  setStepStatus(3, 'processing')
  resumePrompt.value = false

  try {
    const workDir = projectDir.value
    const videoPath = videoFile.value.path
    const subtitles = translatedSubtitles.value

    // Initialize job
    const jobId = await dubbing.initJob({
      projectDir: workDir,
      videoPath,
      subtitleCount: subtitles.length,
      referenceMode: referenceMode.value,
      referenceAudioPath: referenceMode.value === 'custom' ? customAudioPath.value : undefined,
      ttsPluginId: selectedPluginId.value,
    })

    // Stage 1: Preprocess
    const preprocessed = await dubbing.runPreprocess(jobId, subtitles)
    dubbing.preprocessedTexts.value = preprocessed

    // Stage 2: Media separation (needed for clone mode)
    let vocalPath = ''
    let silentVideoPath = ''
    if (referenceMode.value === 'clone' || true) {
      const mediaResult = await dubbing.runMediaSeparation(jobId, videoPath, workDir)
      vocalPath = mediaResult.vocalAudioPath
      silentVideoPath = mediaResult.silentVideoPath
      dubbing.vocalAudioPath.value = vocalPath
      dubbing.silentVideoPath.value = silentVideoPath
    }

    // Stage 3: Reference generation
    await dubbing.runReferenceGeneration({
      jobId,
      referenceMode: referenceMode.value,
      vocalAudioPath: vocalPath || undefined,
      customAudioPath: referenceMode.value === 'custom' ? customAudioPath.value : undefined,
      subtitleEntries: subtitles,
      workDir,
    })

    // Init TTS items in DB
    await dubbing.initTtsItems(jobId, subtitles, preprocessed)

    // Stage 4: TTS generation
    if (referenceMode.value === 'none') {
      // Built-in NCN mode
      await dubbing.runTtsGeneration(jobId, workDir, { ncnVoiceId: ncnVoiceId.value })
    } else if (selectedPluginId.value) {
      await dubbing.runTtsGeneration(jobId, workDir, { pluginId: selectedPluginId.value })
    } else {
      throw new Error('未选择 TTS 插件')
    }

    // Stages 5+6: Alignment + compose
    const outputPath = `${workDir}/output_dubbed.mp4`
    await dubbing.runAlignmentAndCompose({
      jobId,
      silentVideoPath,
      workDir,
      outputPath,
    })
    dubbing.outputPath.value = outputPath

    setStepStatus(3, 'completed')
  } catch (err) {
    console.error('[StepDubbing] pipeline error:', err)
    dubbing.isRunning.value = false
    setStepStatus(3, 'ready')
    return
  }
  dubbing.isRunning.value = false
}

async function onResume() {
  resumePrompt.value = false
  await startDubbing()
}

async function onStartFresh() {
  await dubbing.resetJob()
  resumePrompt.value = false
  dubbing.resetState()
}

function onCancel() {
  dubbing.cancel()
}
</script>

<template>
  <div class="step-dubbing">
    <!-- Resume prompt -->
    <div v-if="resumePrompt" class="resume-card">
      <div class="resume-card__icon">↺</div>
      <div class="resume-card__body">
        <p class="resume-card__title">检测到未完成的配音任务</p>
        <p class="resume-card__desc">是否继续上次未完成的配音？</p>
      </div>
      <div class="resume-card__actions">
        <button class="btn btn--primary" @click="onResume">继续</button>
        <button class="btn btn--secondary" @click="onStartFresh">重新开始</button>
      </div>
    </div>

    <!-- Config panel -->
    <template v-else-if="isConfigPhase && !isCompleted">
      <div class="panel">
        <p class="panel__title">参考音频模式</p>
        <ReferenceAudioPicker
          v-model="referenceMode"
          :custom-audio-path="customAudioPath"
          :ncn-voice-id="ncnVoiceId"
          @update:custom-audio-path="customAudioPath = $event"
          @update:ncn-voice-id="ncnVoiceId = $event"
        />
      </div>

      <div v-if="referenceMode !== 'none'" class="panel">
        <p class="panel__title">TTS 提供商</p>
        <TtsPluginSelector v-model="selectedPluginId" />
      </div>

      <button
        class="btn btn--primary btn--start"
        :disabled="
          (referenceMode === 'none' && !ncnVoiceId) ||
          (referenceMode !== 'none' && !selectedPluginId) ||
          (referenceMode === 'custom' && !customAudioPath)
        "
        @click="startDubbing"
      >
        开始配音
      </button>
    </template>

    <!-- Running panel -->
    <template v-else-if="isRunning">
      <DubbingProgress
        :stage-statuses="dubbing.stageStatuses.value"
        :stage-progress="dubbing.stageProgress.value"
        :tts-total="translatedSubtitles.length || undefined"
        :tts-completed="ttsCompletedCount || undefined"
        :current-message="dubbing.currentMessage.value"
      />
      <button class="btn btn--danger" @click="onCancel">取消</button>
    </template>

    <!-- Completed panel -->
    <div v-else-if="isCompleted" class="done-panel">
      <div class="done-card">
        <div class="done-card__icon">✓</div>
        <p class="done-card__title">配音完成</p>
        <p class="done-card__path">{{ dubbing.outputPath.value }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.step-dubbing {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 600px;
  margin: 0 auto;
  width: 100%;
}

.resume-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: var(--bg-elevated);
  border: 1px solid var(--accent);
  border-radius: 12px;
}

.resume-card__icon {
  font-size: 24px;
  color: var(--accent);
  flex-shrink: 0;
}

.resume-card__body {
  flex: 1;
}

.resume-card__title {
  margin: 0 0 4px;
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
}

.resume-card__desc {
  margin: 0;
  font-size: 13px;
  color: var(--text-muted);
}

.resume-card__actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.panel {
  padding: 18px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.panel__title {
  margin: 0;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.btn {
  padding: 8px 20px;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  border: none;
  transition: all 0.15s ease;
}

.btn--primary {
  background: var(--accent);
  color: #fff;
}

.btn--primary:hover:not(:disabled) {
  background: var(--accent-hover);
}

.btn--secondary {
  background: var(--bg-hover);
  color: var(--text-secondary);
  border: 1px solid var(--border);
}

.btn--secondary:hover {
  background: var(--bg-elevated);
}

.btn--danger {
  background: var(--status-error-subtle);
  color: var(--status-error);
  border: 1px solid var(--status-error);
  align-self: flex-start;
}

.btn--start {
  align-self: flex-start;
}

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.done-panel {
  width: 100%;
}

.done-card {
  padding: 32px 24px;
  background: var(--status-success-subtle);
  border: 1px solid var(--status-success);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.done-card__icon {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--status-success);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: 700;
}

.done-card__title {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
}

.done-card__path {
  margin: 0;
  font-size: 12px;
  color: var(--text-muted);
  word-break: break-all;
  text-align: center;
}
</style>
