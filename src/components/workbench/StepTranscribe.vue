<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useWorkbench } from '@/composables/useWorkbench'
import { useTranscriptionSettings } from '@/composables/useTranscriptionSettings'
import { LANGUAGES } from '@/types/workbench'
import ProgressBar from './ProgressBar.vue'

const {
  videoFile, sourceLanguage, stepStatuses, setStepStatus,
  originalSubtitles, progress, projectDir,
} = useWorkbench()

const { activeProvider, transcriptionSettings, validateActive } = useTranscriptionSettings()

const showAdvanced = ref(false)
const errorMessage = ref('')

let cancelFlag = false

async function startProcessing() {
  const result = validateActive()
  if (!result.valid) {
    errorMessage.value = Object.values(result.errors)[0] ?? '配置校验失败，请检查设置'
    return
  }
  if (!videoFile.value) {
    errorMessage.value = '请先上传视频文件'
    return
  }

  errorMessage.value = ''
  cancelFlag = false
  setStepStatus(1, 'processing')

  try {
    // Step 1: Create project & cache directories
    progress.value = { phase: '初始化', percent: 5, message: '创建项目目录...' }
    const stem = videoFile.value.name.replace(/\.[^.]+$/, '')
    const { projectDir: dir, cacheDir } = await invoke<{ projectDir: string; cacheDir: string }>(
      'cmd_create_project_dir', { videoStem: stem }
    )
    projectDir.value = dir

    // Step 2: Extract audio into cache dir
    progress.value = { phase: '提取音频', percent: 10, message: '提取音频中...' }
    const audioPath = cacheDir + '/audio.wav'
    await invoke('cmd_extract_audio', {
      videoPath: videoFile.value.path,
      outputPath: audioPath,
    })

    if (cancelFlag) { resetToReady(); return }
    progress.value = { phase: '转录', percent: 40, message: '上传并转录中...' }

    // Step 3: Transcribe
    const providerId = transcriptionSettings.value.activeProviderId
    let subtitlesJson: string

    if (providerId === 'bcut') {
      const cfg = transcriptionSettings.value.configs.bcut
      subtitlesJson = await invoke<string>('cmd_transcribe_bcut', {
        audioPath,
        language: cfg.language,
      })
    } else {
      const isPaid = providerId === 'elevenlabs-paid'
      const cfg = isPaid
        ? transcriptionSettings.value.configs['elevenlabs-paid']
        : transcriptionSettings.value.configs['elevenlabs-free']
      subtitlesJson = await invoke<string>('cmd_transcribe_elevenlabs', {
        audioPath,
        modelId: cfg.modelId,
        language: cfg.language,
        numSpeakers: cfg.numSpeakers,
        tagAudioEvents: cfg.tagAudioEvents,
        apiKey: isPaid ? transcriptionSettings.value.configs['elevenlabs-paid'].apiKey : '',
      })
    }

    if (cancelFlag) { resetToReady(); return }
    progress.value = { phase: '保存', percent: 85, message: '处理字幕...' }

    // Step 4: Save subtitles and clean up cache
    await invoke('cmd_save_subtitles', { projectDir: dir, cacheDir, subtitlesJson })

    // Done
    originalSubtitles.value = JSON.parse(subtitlesJson)
    setStepStatus(1, 'completed')
    progress.value = { phase: '', percent: 100, message: '' }

  } catch (err) {
    setStepStatus(1, 'ready')
    errorMessage.value = String(err)
    progress.value = { phase: '', percent: 0, message: '' }
  }
}

function cancelProcessing() {
  cancelFlag = true
}

function resetToReady() {
  setStepStatus(1, 'ready')
  progress.value = { phase: '', percent: 0, message: '' }
}
</script>

<template>
  <div class="step-transcribe">
    <!-- Config state -->
    <div v-if="stepStatuses[1] === 'ready'" class="config-panel">
      <div class="field">
        <label class="field-label">源语言</label>
        <select v-model="sourceLanguage" class="select">
          <option v-for="l in LANGUAGES" :key="l.code" :value="l.code">{{ l.label }}</option>
        </select>
      </div>

      <button class="advanced-toggle" @click="showAdvanced = !showAdvanced">
        {{ showAdvanced ? '收起' : '高级选项' }}
      </button>

      <div v-if="showAdvanced" class="advanced-options">
        <div class="active-provider-hint">
          <div class="active-provider-info">
            <span class="active-provider-name">{{ activeProvider.name }}</span>
            <span class="active-provider-desc">{{ activeProvider.description }}</span>
          </div>
          <router-link to="/settings" class="go-settings-link">在设置中更改 →</router-link>
        </div>
      </div>

      <p v-if="errorMessage" class="validation-error">{{ errorMessage }}</p>

      <button class="btn btn--primary" @click="startProcessing">开始转录</button>
    </div>

    <!-- Processing state -->
    <div v-else-if="stepStatuses[1] === 'processing'" class="progress-panel">
      <div class="progress-card">
        <p class="progress-phase">{{ progress.message }}</p>
        <ProgressBar :percent="progress.percent" :label="progress.phase" show-percent />
        <button class="btn btn--secondary" @click="cancelProcessing">取消</button>
      </div>
    </div>

    <!-- Completed state -->
    <div v-else-if="stepStatuses[1] === 'completed'" class="done-panel">
      <div class="done-card">
        <div class="done-card__icon">✓</div>
        <p class="done-card__title">转录完成</p>
        <div class="done-card__meta">
          <span>{{ originalSubtitles.length }} 条字幕</span>
          <span>{{ sourceLanguage }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.step-transcribe {
  display: flex;
  justify-content: center;
  padding-top: 24px;
}

.config-panel {
  max-width: 480px;
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.field-label {
  font-size: 13px;
  color: var(--text-secondary);
}

.select {
  padding: 8px 12px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 14px;
  outline: none;
  transition: border-color 0.15s ease;
}

.select:focus {
  border-color: var(--accent);
}

.advanced-toggle {
  background: none;
  border: none;
  color: var(--accent);
  font-size: 13px;
  cursor: pointer;
  padding: 0;
  text-align: left;
}

.advanced-options {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

/* Active provider hint */
.active-provider-hint {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.active-provider-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.active-provider-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.active-provider-desc {
  font-size: 12px;
  color: var(--text-secondary);
}

.go-settings-link {
  font-size: 13px;
  color: var(--accent);
  text-decoration: none;
  white-space: nowrap;
  flex-shrink: 0;
}

.go-settings-link:hover {
  text-decoration: underline;
}

.validation-error {
  margin: 0;
  font-size: 13px;
  color: var(--status-error);
  padding: 10px 14px;
  background: var(--status-error-subtle, rgba(239, 68, 68, 0.08));
  border: 1px solid var(--status-error);
  border-radius: 8px;
}

.progress-panel {
  max-width: 480px;
  width: 100%;
}

.progress-card {
  padding: 32px 24px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.progress-phase {
  margin: 0;
  font-size: 15px;
  color: var(--text-primary);
}

.done-panel {
  max-width: 480px;
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

.done-card__meta {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--text-secondary);
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

.btn--primary:hover {
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
</style>
