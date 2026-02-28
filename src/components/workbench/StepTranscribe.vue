<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useWorkbench } from '@/composables/useWorkbench'
import { useTranscriptionSettings } from '@/composables/useTranscriptionSettings'
import { LANGUAGES } from '@/types/workbench'
import ProgressBar from './ProgressBar.vue'
import SubtitleRow from './SubtitleRow.vue'

const {
  videoFile, sourceLanguage, stepStatuses, setStepStatus,
  originalSubtitles, progress, projectDir, workbenchTaskId, createTask, saveProgress,
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
    await createTask()

    // Step 2: Extract audio into cache dir
    progress.value = { phase: '提取音频', percent: 10, message: '提取音频中...' }
    const audioPath = cacheDir + '/audio.mp3'
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
      subtitlesJson = await invoke<string>('cmd_transcribe_bcut', {
        audioPath,
        language: sourceLanguage.value,
      })
    } else {
      const isPaid = providerId === 'elevenlabs-paid'
      const cfg = isPaid
        ? transcriptionSettings.value.configs['elevenlabs-paid']
        : transcriptionSettings.value.configs['elevenlabs-free']
      subtitlesJson = await invoke<string>('cmd_transcribe_elevenlabs', {
        audioPath,
        modelId: cfg.modelId,
        language: sourceLanguage.value,
        numSpeakers: cfg.numSpeakers,
        tagAudioEvents: cfg.tagAudioEvents,
        enableDiarization: isPaid
          ? transcriptionSettings.value.configs['elevenlabs-paid'].enableDiarization
          : false,
        apiKey: isPaid ? transcriptionSettings.value.configs['elevenlabs-paid'].apiKey : '',
      })
    }

    if (cancelFlag) { resetToReady(); return }
    progress.value = { phase: '保存', percent: 85, message: '处理字幕...' }

    // Step 4: Save subtitles and clean up cache
    await invoke('cmd_save_subtitles', { projectDir: dir, cacheDir, subtitlesJson })

    // Done
    originalSubtitles.value = JSON.parse(subtitlesJson)
    await invoke('cmd_save_transcribe_step', {
      taskId: workbenchTaskId.value,
      configJson: JSON.stringify({
        providerId: transcriptionSettings.value.activeProviderId,
        sourceLanguage: sourceLanguage.value,
      }),
      subtitlesPath: `${dir}/subtitles.json`,
      subtitleCount: originalSubtitles.value.length,
    }).catch(() => {})
    setStepStatus(1, 'completed')
    await saveProgress()
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

function onUpdateSubtitle(idx: number, text: string) {
  originalSubtitles.value[idx].text = text
}
</script>

<template>
  <div class="step-transcribe">
    <!-- Config state -->
    <div v-if="stepStatuses[1] === 'ready'" class="step-transcribe__center">
      <div class="config-panel">
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
    </div>

    <!-- Processing state -->
    <div v-else-if="stepStatuses[1] === 'processing'" class="step-transcribe__center">
      <div class="progress-panel">
        <div class="progress-card">
          <p class="progress-phase">{{ progress.message }}</p>
          <ProgressBar :percent="progress.percent" :label="progress.phase" show-percent />
          <button class="btn btn--secondary" @click="cancelProcessing">取消</button>
        </div>
      </div>
    </div>

    <!-- Completed state — full-width card with editable subtitle list -->
    <div v-else-if="stepStatuses[1] === 'completed'" class="transcribe-completed">
      <div class="transcribe-header">
        <div class="transcribe-header__badge">
          <span class="badge-icon">✓</span>
          <span>转录完成</span>
        </div>
        <div class="transcribe-header__meta">
          <span>{{ originalSubtitles.length }} 条字幕</span>
          <span>源语言: {{ sourceLanguage }}</span>
        </div>
        <button class="btn btn--ghost" @click="resetToReady">重新转录</button>
      </div>
      <div class="transcribe-subtitle-list">
        <SubtitleRow
          v-for="(sub, idx) in originalSubtitles"
          :key="sub.id"
          :subtitle="sub"
          editable
          @update="(text) => onUpdateSubtitle(idx, text)"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.step-transcribe {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.step-transcribe__center {
  flex: 1;
  display: flex;
  justify-content: center;
  padding-top: 24px;
  overflow-y: auto;
}

/* ── Config panel ──────────────────────────────────────────────────────── */

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

/* ── Progress panel ────────────────────────────────────────────────────── */

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

/* ── Completed state ───────────────────────────────────────────────────── */

.transcribe-completed {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: 10px;
}

.transcribe-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elevated);
  flex-shrink: 0;
}

.transcribe-header__badge {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  font-weight: 500;
  color: var(--status-success);
}

.badge-icon {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--status-success);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  flex-shrink: 0;
}

.transcribe-header__meta {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--text-secondary);
  flex: 1;
}

.transcribe-subtitle-list {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

/* ── Shared buttons ────────────────────────────────────────────────────── */

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

.btn--ghost {
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border);
  padding: 6px 14px;
  font-size: 13px;
  margin-left: auto;
}

.btn--ghost:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
</style>
