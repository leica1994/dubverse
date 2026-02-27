<script setup lang="ts">
import { ref } from 'vue'
import { useWorkbench } from '@/composables/useWorkbench'
import { MOCK_VOICES } from '@/types/workbench'
import ProgressBar from './ProgressBar.vue'
import IconPlay from '@/components/icons/IconPlay.vue'

const {
  ttsConfig, stepStatuses, setStepStatus, progress,
} = useWorkbench()

const selectedVoice = ref(ttsConfig.value.voiceId || '')
const speed = ref(ttsConfig.value.speed)
const pitch = ref(ttsConfig.value.pitch)

function selectVoice(id: string) {
  selectedVoice.value = id
  ttsConfig.value.voiceId = id
}

function startDubbing() {
  if (!selectedVoice.value) return
  ttsConfig.value = { voiceId: selectedVoice.value, speed: speed.value, pitch: pitch.value }
  setStepStatus(4, 'processing')

  let elapsed = 0
  const total = 3000
  const interval = setInterval(() => {
    elapsed += 50
    progress.value = {
      phase: '配音生成',
      percent: Math.min(100, (elapsed / total) * 100),
      message: '正在生成配音...',
    }
    if (elapsed >= total) {
      clearInterval(interval)
      setStepStatus(4, 'completed')
      progress.value = { phase: '', percent: 100, message: '' }
    }
  }, 50)
}
</script>

<template>
  <div class="step-dubbing">
    <!-- Voice selection + config -->
    <template v-if="stepStatuses[4] === 'ready'">
      <p class="section-title">选择配音声音</p>
      <div class="voice-grid">
        <div
          v-for="voice in MOCK_VOICES"
          :key="voice.id"
          class="voice-card"
          :class="{ 'voice-card--selected': selectedVoice === voice.id }"
          @click="selectVoice(voice.id)"
        >
          <div class="voice-card__icon">
            <IconPlay />
          </div>
          <span class="voice-card__name">{{ voice.name }}</span>
          <span class="voice-card__gender">{{ voice.gender === 'male' ? '男' : '女' }}</span>
        </div>
      </div>

      <div class="sliders">
        <div class="slider-field">
          <label class="field-label">语速: {{ speed.toFixed(1) }}x</label>
          <input type="range" v-model.number="speed" min="0.5" max="2.0" step="0.1" class="range" />
        </div>
        <div class="slider-field">
          <label class="field-label">音调: {{ pitch.toFixed(1) }}x</label>
          <input type="range" v-model.number="pitch" min="0.5" max="2.0" step="0.1" class="range" />
        </div>
      </div>

      <button
        class="btn btn--primary"
        :disabled="!selectedVoice"
        @click="startDubbing"
      >生成配音</button>
    </template>

    <!-- Processing -->
    <div v-else-if="stepStatuses[4] === 'processing'" class="progress-panel">
      <div class="progress-card">
        <p class="progress-phase">{{ progress.message }}</p>
        <ProgressBar :percent="progress.percent" label="配音生成" show-percent />
      </div>
    </div>

    <!-- Completed -->
    <div v-else-if="stepStatuses[4] === 'completed'" class="done-panel">
      <div class="done-card">
        <div class="done-card__icon">✓</div>
        <p class="done-card__title">配音生成完成</p>
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

.section-title {
  margin: 0;
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
}

.voice-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 12px;
}

.voice-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 16px 12px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.voice-card:hover {
  background: var(--bg-hover);
}

.voice-card--selected {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-subtle);
}

.voice-card__icon {
  color: var(--text-muted);
}

.voice-card__name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.voice-card__gender {
  font-size: 12px;
  color: var(--text-muted);
}

.sliders {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.slider-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 13px;
  color: var(--text-secondary);
}

.range {
  width: 100%;
  accent-color: var(--accent);
}

.progress-panel,
.done-panel {
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

.btn {
  padding: 8px 20px;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  border: none;
  transition: all 0.15s ease;
  align-self: flex-start;
}

.btn--primary {
  background: var(--accent);
  color: #fff;
}

.btn--primary:hover {
  background: var(--accent-hover);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
