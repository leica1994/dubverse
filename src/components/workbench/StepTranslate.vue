<script setup lang="ts">
import { ref } from 'vue'
import { useWorkbench } from '@/composables/useWorkbench'
import { LANGUAGES } from '@/types/workbench'
import type { Subtitle } from '@/types/workbench'
import ProgressBar from './ProgressBar.vue'

const {
  targetLanguage, sourceLanguage, stepStatuses, setStepStatus,
  originalSubtitles, translatedSubtitles, progress,
} = useWorkbench()

const translateEngine = ref('google')
let cancelFlag = false

function startTranslation() {
  cancelFlag = false
  setStepStatus(2, 'processing')

  let elapsed = 0
  const total = 2000

  const interval = setInterval(() => {
    if (cancelFlag) {
      clearInterval(interval)
      setStepStatus(2, 'ready')
      progress.value = { phase: '', percent: 0, message: '' }
      return
    }

    elapsed += 50
    progress.value = {
      phase: '翻译处理',
      percent: Math.min(100, (elapsed / total) * 100),
      message: '翻译处理中...',
    }

    if (elapsed >= total) {
      clearInterval(interval)
      generateMockTranslation()
      setStepStatus(2, 'completed')
      progress.value = { phase: '', percent: 100, message: '' }
    }
  }, 50)
}

function cancelTranslation() {
  cancelFlag = true
}

function generateMockTranslation() {
  const mockTranslated: Subtitle[] = [
    { id: 1, startTime: 0, endTime: 3.2, text: 'Welcome to our channel' },
    { id: 2, startTime: 3.5, endTime: 6.8, text: "Today we're going to discuss an interesting topic" },
    { id: 3, startTime: 7.1, endTime: 10.5, text: 'About the application of AI in video production' },
    { id: 4, startTime: 11.0, endTime: 14.2, text: "Let's get started" },
    { id: 5, startTime: 15.0, endTime: 18.5, text: "First let's look at the basic concepts" },
  ]
  translatedSubtitles.value = mockTranslated
}
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
        <label class="field-label">翻译引擎</label>
        <select v-model="translateEngine" class="select">
          <option value="google">Google Translate</option>
          <option value="deepl">DeepL</option>
          <option value="openai">OpenAI</option>
        </select>
      </div>

      <button class="btn btn--primary" @click="startTranslation">开始翻译</button>
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
</style>
