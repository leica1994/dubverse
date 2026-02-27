<script setup lang="ts">
import { ref } from 'vue'
import { useWorkbench } from '@/composables/useWorkbench'
import { LANGUAGES } from '@/types/workbench'
import type { Subtitle } from '@/types/workbench'
import ProgressBar from './ProgressBar.vue'

const {
  sourceLanguage, stepStatuses, setStepStatus,
  originalSubtitles, progress,
} = useWorkbench()

const showAdvanced = ref(false)
const transcribeModel = ref('whisper-large-v3')

let cancelFlag = false

function startProcessing() {
  cancelFlag = false
  setStepStatus(1, 'processing')

  const phases = [
    { phase: '音频提取', duration: 1500 },
    { phase: '语音转录', duration: 2500 },
  ]

  let phaseIndex = 0
  let elapsed = 0
  const totalDuration = phases.reduce((s, p) => s + p.duration, 0)

  const interval = setInterval(() => {
    if (cancelFlag) {
      clearInterval(interval)
      setStepStatus(1, 'ready')
      progress.value = { phase: '', percent: 0, message: '' }
      return
    }

    elapsed += 50
    let acc = 0
    for (let i = 0; i < phases.length; i++) {
      acc += phases[i].duration
      if (elapsed <= acc) { phaseIndex = i; break }
    }

    const pct = Math.min(100, (elapsed / totalDuration) * 100)
    progress.value = {
      phase: phases[phaseIndex].phase,
      percent: pct,
      message: `${phases[phaseIndex].phase}中...`,
    }

    if (elapsed >= totalDuration) {
      clearInterval(interval)
      generateMockSubtitles()
      setStepStatus(1, 'completed')
      progress.value = { phase: '', percent: 100, message: '' }
    }
  }, 50)
}

function cancelProcessing() {
  cancelFlag = true
}

function generateMockSubtitles() {
  const mockOriginal: Subtitle[] = [
    { id: 1, startTime: 0, endTime: 3.2, text: '欢迎来到我们的频道' },
    { id: 2, startTime: 3.5, endTime: 6.8, text: '今天我们要讨论一个有趣的话题' },
    { id: 3, startTime: 7.1, endTime: 10.5, text: '关于人工智能在视频制作中的应用' },
    { id: 4, startTime: 11.0, endTime: 14.2, text: '让我们开始吧' },
    { id: 5, startTime: 15.0, endTime: 18.5, text: '首先我们来看一下基本概念' },
  ]
  originalSubtitles.value = mockOriginal
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
        <div class="field">
          <label class="field-label">转录模型</label>
          <select v-model="transcribeModel" class="select">
            <option value="whisper-large-v3">Whisper Large V3</option>
            <option value="whisper-medium">Whisper Medium</option>
            <option value="whisper-small">Whisper Small</option>
          </select>
        </div>
      </div>

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
