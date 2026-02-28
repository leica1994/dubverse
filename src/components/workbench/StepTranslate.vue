<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { useWorkbench } from '@/composables/useWorkbench'
import { useTranslationSettings } from '@/composables/useTranslationSettings'
import { useAiConfigs } from '@/composables/useAiConfigs'
import { LANGUAGES } from '@/types/workbench'
import ProgressBar from './ProgressBar.vue'

const {
  targetLanguage, sourceLanguage, stepStatuses, setStepStatus,
  originalSubtitles, translatedSubtitles, progress, projectDir,
} = useWorkbench()

const { translationSettings } = useTranslationSettings()
const { aiConfigs } = useAiConfigs()

const defaultConfig = computed(() => aiConfigs.value.find(c => c.isDefault))
const errorMsg = ref('')
let unlisten: UnlistenFn | null = null

async function startTranslation() {
  if (!defaultConfig.value) {
    errorMsg.value = '请先在设置中添加并设为默认 AI 配置'
    return
  }
  errorMsg.value = ''
  setStepStatus(2, 'processing')

  // Listen for progress events
  unlisten = await listen<{ phase: string; batch: number; totalBatches: number; skipped: number; percent: number; message: string }>('translate:progress', (event) => {
    progress.value = {
      phase: event.payload.phase,
      percent: event.payload.percent,
      message: event.payload.message,
    }
  })

  const ts = translationSettings.value
  try {
    const result = await invoke<Array<{ id: number; startTime: number; endTime: number; text: string }>>('cmd_start_translation', {
      subtitles: originalSubtitles.value,
      projectDir: projectDir.value,
      targetLanguage: targetLanguage.value,
      correction: ts.correction,
      optimization: ts.optimization,
      promptType: ts.promptType,
      batchSize: ts.batchSize,
      worldBuilding: ts.worldBuilding,
      writingStyle: ts.writingStyle,
      glossary: ts.glossary,
      forbidden: ts.forbidden,
      examples: ts.examples,
      customPrompt: ts.customPrompt,
    })
    translatedSubtitles.value = result
    setStepStatus(2, 'completed')
    progress.value = { phase: '', percent: 100, message: '' }
  } catch (err) {
    const msg = String(err)
    if (msg.includes('已取消')) {
      setStepStatus(2, 'ready')
      progress.value = { phase: '', percent: 0, message: '' }
    } else {
      errorMsg.value = msg
      setStepStatus(2, 'ready')
      progress.value = { phase: '', percent: 0, message: '' }
    }
  } finally {
    unlisten?.()
    unlisten = null
  }
}

async function cancelTranslation() {
  await invoke('cmd_cancel_translation')
}

onUnmounted(() => {
  unlisten?.()
})
</script>

<template>
  <div class="step-translate">
    <!-- Config state -->
    <div v-if="stepStatuses[2] === 'ready'" class="config-panel">
      <div class="field">
        <label class="field-label">目标语言</label>
        <select v-model="targetLanguage" class="select">
          <option v-for="l in LANGUAGES" :key="l.code" :value="l.code">{{ l.label }}</option>
        </select>
      </div>

      <div class="field">
        <label class="field-label">AI 模型</label>
        <div v-if="defaultConfig" class="ai-info">
          <span class="ai-info__name">{{ defaultConfig.title }}</span>
          <span class="ai-info__meta">{{ defaultConfig.model }}</span>
        </div>
        <span v-else class="field-warn">未配置默认 AI 模型，请先在设置中添加</span>
      </div>

      <p v-if="errorMsg" class="error-msg">{{ errorMsg }}</p>

      <button class="btn btn--primary" @click="startTranslation" :disabled="!defaultConfig">开始翻译</button>
    </div>

    <!-- Processing state -->
    <div v-else-if="stepStatuses[2] === 'processing'" class="progress-panel">
      <div class="progress-card">
        <p class="progress-phase">{{ progress.message }}</p>
        <ProgressBar :percent="progress.percent" :label="progress.phase" show-percent />
        <button class="btn btn--secondary" @click="cancelTranslation">取消</button>
      </div>
    </div>

    <!-- Completed state -->
    <div v-else-if="stepStatuses[2] === 'completed'" class="done-panel">
      <div class="done-card">
        <div class="done-card__icon">✓</div>
        <p class="done-card__title">翻译完成</p>
        <div class="done-card__meta">
          <span>{{ originalSubtitles.length }} 条字幕</span>
          <span>{{ sourceLanguage }} → {{ targetLanguage }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.step-translate {
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

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
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

.ai-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 10px 12px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 8px;
}

.ai-info__name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.ai-info__meta {
  font-size: 12px;
  color: var(--text-muted);
}

.field-warn {
  font-size: 13px;
  color: var(--status-warning);
}

.error-msg {
  margin: 0;
  font-size: 13px;
  color: var(--status-error);
}

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
