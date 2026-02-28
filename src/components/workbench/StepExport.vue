<script setup lang="ts">
import { ref } from 'vue'
import { useWorkbench } from '@/composables/useWorkbench'
import ProgressBar from './ProgressBar.vue'

const {
  exportConfig, stepStatuses, setStepStatus, progress,
  videoFile, originalSubtitles, ttsConfig, sourceLanguage, targetLanguage,
  resetWorkbench,
} = useWorkbench()

const exportDone = ref(false)

function startExport() {
  setStepStatus(4, 'processing')
  exportDone.value = false

  let elapsed = 0
  const total = 2500
  const interval = setInterval(() => {
    elapsed += 50
    progress.value = {
      phase: '导出',
      percent: Math.min(100, (elapsed / total) * 100),
      message: '正在导出视频...',
    }
    if (elapsed >= total) {
      clearInterval(interval)
      setStepStatus(4, 'completed')
      exportDone.value = true
      progress.value = { phase: '', percent: 100, message: '' }
    }
  }, 50)
}

function onNewTask() {
  exportDone.value = false
  resetWorkbench()
}
</script>

<template>
  <div class="step-export">
    <!-- Config state -->
    <template v-if="stepStatuses[4] === 'ready'">
      <div class="export-settings">
        <p class="section-title">导出设置</p>
        <div class="field-row">
          <div class="field">
            <label class="field-label">格式</label>
            <select v-model="exportConfig.format" class="select">
              <option value="mp4">MP4</option>
              <option value="mkv">MKV</option>
              <option value="webm">WebM</option>
            </select>
          </div>
          <div class="field">
            <label class="field-label">质量</label>
            <select v-model="exportConfig.quality" class="select">
              <option value="high">高质量</option>
              <option value="medium">中等</option>
              <option value="low">低质量</option>
            </select>
          </div>
        </div>
        <div class="field">
          <label class="field-label">保存路径</label>
          <div class="path-input">
            <input
              v-model="exportConfig.outputPath"
              class="input"
              placeholder="选择保存位置..."
              readonly
            />
            <button class="btn btn--secondary" @click="exportConfig.outputPath = 'D:/output/dubverse'">
              浏览
            </button>
          </div>
        </div>
      </div>

      <div class="summary-card">
        <p class="section-title">处理摘要</p>
        <div class="summary-grid">
          <div class="summary-item">
            <span class="summary-item__label">源视频</span>
            <span class="summary-item__value">{{ videoFile?.name || '-' }}</span>
          </div>
          <div class="summary-item">
            <span class="summary-item__label">字幕数</span>
            <span class="summary-item__value">{{ originalSubtitles.length }} 条</span>
          </div>
          <div class="summary-item">
            <span class="summary-item__label">配音声音</span>
            <span class="summary-item__value">{{ ttsConfig.voiceId || '-' }}</span>
          </div>
          <div class="summary-item">
            <span class="summary-item__label">语言对</span>
            <span class="summary-item__value">{{ sourceLanguage }} → {{ targetLanguage }}</span>
          </div>
        </div>
      </div>

      <button class="btn btn--primary" @click="startExport">开始导出</button>
    </template>

    <!-- Processing -->
    <div v-else-if="stepStatuses[4] === 'processing'" class="progress-panel">
      <div class="progress-card">
        <p class="progress-phase">{{ progress.message }}</p>
        <ProgressBar :percent="progress.percent" label="导出" show-percent />
      </div>
    </div>

    <!-- Completed -->
    <div v-else-if="exportDone" class="done-panel">
      <div class="done-card">
        <div class="done-card__icon">✓</div>
        <p class="done-card__title">导出完成</p>
        <div class="done-card__actions">
          <button class="btn btn--secondary">打开文件位置</button>
          <button class="btn btn--secondary">打开文件</button>
          <button class="btn btn--primary" @click="onNewTask">新建任务</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.step-export {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 560px;
  margin: 0 auto;
  width: 100%;
}

.section-title {
  margin: 0 0 12px;
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
}

.export-settings {
  padding: 20px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 12px;
}

.field-row {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
}

.field {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 13px;
  color: var(--text-secondary);
}

.select,
.input {
  padding: 8px 12px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 14px;
  outline: none;
  transition: border-color 0.15s ease;
}

.select:focus,
.input:focus {
  border-color: var(--accent);
}

.path-input {
  display: flex;
  gap: 8px;
}

.path-input .input {
  flex: 1;
}

.summary-card {
  padding: 20px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 12px;
}

.summary-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.summary-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.summary-item__label {
  font-size: 12px;
  color: var(--text-muted);
}

.summary-item__value {
  font-size: 14px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
  gap: 12px;
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

.done-card__actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
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
