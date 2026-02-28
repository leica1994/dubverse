<script setup lang="ts">
import { ref } from 'vue'
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener'
import { useWorkbench } from '@/composables/useWorkbench'
import { useDubbing } from '@/composables/useDubbing'

const {
  videoFile, originalSubtitles, translatedSubtitles,
  sourceLanguage, targetLanguage,
  resetWorkbench,
} = useWorkbench()

const dubbing = useDubbing()

const isCopying = ref(false)
const copyDone = ref(false)
const copyError = ref('')
const savedPath = ref('')

const outputPath = dubbing.outputPath

async function openInFolder() {
  const path = savedPath.value || outputPath.value
  if (!path) return
  await revealItemInDir(path)
}

async function saveToPath() {
  const src = outputPath.value
  if (!src) return
  isCopying.value = true
  copyError.value = ''
  try {
    await openPath(src)
    savedPath.value = src
    copyDone.value = true
  } catch (err) {
    copyError.value = String(err)
  } finally {
    isCopying.value = false
  }
}

function onNewTask() {
  copyDone.value = false
  copyError.value = ''
  dubbing.resetState()
  resetWorkbench()
}
</script>

<template>
  <div class="step-export">
    <!-- Summary -->
    <div class="summary-card">
      <p class="section-title">处理摘要</p>
      <div class="summary-grid">
        <div class="summary-item">
          <span class="summary-label">源视频</span>
          <span class="summary-value">{{ videoFile?.name || '-' }}</span>
        </div>
        <div class="summary-item">
          <span class="summary-label">字幕数</span>
          <span class="summary-value">{{ translatedSubtitles.length || originalSubtitles.length }} 条</span>
        </div>
        <div class="summary-item">
          <span class="summary-label">语言对</span>
          <span class="summary-value">{{ sourceLanguage }} → {{ targetLanguage }}</span>
        </div>
        <div class="summary-item">
          <span class="summary-label">输出文件</span>
          <span class="summary-value output-path">{{ outputPath || '配音未完成' }}</span>
        </div>
      </div>
    </div>

    <!-- Actions -->
    <div v-if="outputPath" class="action-card">
      <p class="section-title">导出操作</p>
      <div class="action-btns">
        <button class="btn btn--secondary" @click="openInFolder">
          在文件夹中显示
        </button>
        <button class="btn btn--secondary" :disabled="isCopying" @click="saveToPath">
          {{ isCopying ? '打开中...' : '打开文件' }}
        </button>
      </div>
      <div v-if="copyError" class="error-msg">{{ copyError }}</div>
    </div>

    <div v-else class="no-output-hint">
      尚未完成配音，请先完成 Step 3 配音步骤。
    </div>

    <button class="btn btn--primary btn--new-task" @click="onNewTask">
      新建任务
    </button>
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

.summary-card,
.action-card {
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

.summary-label {
  font-size: 12px;
  color: var(--text-muted);
}

.summary-value {
  font-size: 14px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.output-path {
  font-size: 12px;
  word-break: break-all;
  white-space: normal;
}

.action-btns {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.error-msg {
  margin-top: 10px;
  font-size: 12px;
  color: var(--status-error);
}

.no-output-hint {
  padding: 16px 20px;
  background: var(--bg-elevated);
  border: 1px dashed var(--border);
  border-radius: 12px;
  font-size: 14px;
  color: var(--text-muted);
  text-align: center;
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

.btn--secondary:hover:not(:disabled) {
  background: var(--bg-elevated);
}

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn--new-task {
  align-self: flex-start;
}
</style>
