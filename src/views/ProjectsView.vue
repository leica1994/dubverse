<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useWorkbenchTasks } from '@/composables/useWorkbenchTasks'
import { useWorkbench } from '@/composables/useWorkbench'
import { STEP_LABELS, LANGUAGES, TARGET_LANGUAGES } from '@/types/workbench'
import type { WorkbenchTaskListItem, WorkbenchTaskFull, StepStatus } from '@/types/workbench'
const router = useRouter()
const { tasks, isLoading, loadTasks, getTaskFull, deleteTask } = useWorkbenchTasks()
const { restoreTask, resetWorkbench } = useWorkbench()

const detailTask = ref<WorkbenchTaskFull | null>(null)
const showDetail = ref(false)
const deletingId = ref<string | null>(null)

onMounted(() => {
  loadTasks()
})

// ── Formatting helpers ───────────────────────────────────────────────────────

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${m}:${String(s).padStart(2, '0')}`
}

function formatDate(iso: string): string {
  try {
    const d = new Date(iso)
    const month = d.getMonth() + 1
    const day = d.getDate()
    const hour = String(d.getHours()).padStart(2, '0')
    const min = String(d.getMinutes()).padStart(2, '0')
    return `${month}月${day}日 ${hour}:${min}`
  } catch {
    return iso
  }
}

function langLabel(code: string): string {
  const all = [...LANGUAGES, ...TARGET_LANGUAGES]
  return all.find((l) => l.code === code)?.label ?? code
}

function stepStatusColor(s: StepStatus): string {
  if (s === 'completed') return 'var(--status-success)'
  if (s === 'ready') return 'var(--accent)'
  if (s === 'processing') return 'var(--accent)'
  if (s === 'error') return 'var(--status-error)'
  return 'var(--border)'
}

function stepStatusFilled(s: StepStatus): boolean {
  return s === 'completed' || s === 'ready' || s === 'processing'
}

function currentStepLabel(task: WorkbenchTaskListItem): string {
  if (task.status === 'completed') return '已完成'
  const step = task.currentStep
  const label = STEP_LABELS[step] ?? ''
  const status = task.stepStatuses[step]
  if (status === 'processing') return `处理中·${label}`
  return `进行中·Step ${step + 1}: ${label}`
}

const totalCount = computed(() => tasks.value.length)

// ── Actions ──────────────────────────────────────────────────────────────────

async function openDetail(task: WorkbenchTaskListItem) {
  detailTask.value = await getTaskFull(task.id)
  showDetail.value = true
}

function closeDetail() {
  showDetail.value = false
  detailTask.value = null
}

async function continueTask(task: WorkbenchTaskListItem, event: MouseEvent) {
  event.stopPropagation()
  const full = await getTaskFull(task.id)
  if (!full) return
  resetWorkbench()
  await restoreTask(full)
  router.push('/')
}

async function onDelete(task: WorkbenchTaskListItem, event: MouseEvent) {
  event.stopPropagation()
  deletingId.value = task.id
  try {
    await deleteTask(task.id)
  } finally {
    deletingId.value = null
  }
}

async function continueFromDetail() {
  if (!detailTask.value) return
  const full = detailTask.value
  showDetail.value = false
  resetWorkbench()
  await restoreTask(full)
  router.push('/')
}

function parseConfigJson(json: string): Record<string, string> {
  try {
    return JSON.parse(json) as Record<string, string>
  } catch {
    return {}
  }
}
</script>

<template>
  <div class="projects-view">
    <!-- Header -->
    <div class="projects-header">
      <h2 class="projects-title">所有项目</h2>
      <span v-if="!isLoading" class="projects-count">{{ totalCount }} 个任务</span>
    </div>

    <!-- Loading -->
    <div v-if="isLoading" class="projects-loading">
      <div class="spinner"></div>
      <span>加载中...</span>
    </div>

    <!-- Empty state -->
    <div v-else-if="tasks.length === 0" class="projects-empty">
      <div class="projects-empty__icon">🎬</div>
      <p class="projects-empty__text">还没有任何项目</p>
      <p class="projects-empty__sub">在工作台完成转录后，任务会自动保存到这里</p>
    </div>

    <!-- Task list -->
    <div v-else class="projects-list">
      <div
        v-for="task in tasks"
        :key="task.id"
        class="task-card"
        @click="openDetail(task)"
      >
        <!-- Card header row -->
        <div class="task-card__header">
          <div class="task-card__title-row">
            <span class="task-card__icon">🎬</span>
            <span class="task-card__name">{{ task.videoName }}</span>
            <span class="task-card__duration">{{ formatDuration(task.videoDuration) }}</span>
            <span class="task-card__lang">{{ langLabel(task.sourceLanguage) }} → {{ langLabel(task.targetLanguage) }}</span>
          </div>
          <span class="task-card__date">{{ formatDate(task.createdAt) }}</span>
        </div>

        <!-- Step progress dots -->
        <div class="task-card__progress">
          <div class="step-dots">
            <div
              v-for="(status, idx) in task.stepStatuses"
              :key="idx"
              class="step-dot"
              :class="{ 'step-dot--filled': stepStatusFilled(status as StepStatus) }"
              :style="{ '--dot-color': stepStatusColor(status as StepStatus) }"
              :title="STEP_LABELS[idx]"
            ></div>
          </div>
          <span
            class="task-card__status-label"
            :class="task.status === 'completed' ? 'task-card__status-label--done' : ''"
          >
            {{ currentStepLabel(task) }}
          </span>
        </div>

        <!-- Actions -->
        <div class="task-card__actions" @click.stop>
          <button
            v-if="task.status !== 'completed'"
            class="btn btn--primary btn--sm"
            @click="continueTask(task, $event)"
          >
            继续
          </button>
          <button
            v-else
            class="btn btn--secondary btn--sm"
            @click="continueTask(task, $event)"
          >
            查看
          </button>
          <button
            class="btn btn--danger btn--sm"
            :disabled="deletingId === task.id"
            @click="onDelete(task, $event)"
          >
            {{ deletingId === task.id ? '删除中...' : '删除' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Detail modal -->
    <Teleport to="body">
      <div v-if="showDetail && detailTask" class="modal-overlay" @click.self="closeDetail">
        <div class="modal">
          <div class="modal__header">
            <span class="modal__title">{{ detailTask.name }}</span>
            <button class="modal__close" @click="closeDetail">✕</button>
          </div>

          <div class="modal__body">
            <!-- Video info -->
            <div class="detail-section">
              <div class="detail-row">
                <span class="detail-label">视频文件</span>
                <span class="detail-value">{{ detailTask.videoName }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">时长</span>
                <span class="detail-value">{{ formatDuration(detailTask.videoDuration) }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">语言</span>
                <span class="detail-value">{{ langLabel(detailTask.sourceLanguage) }} → {{ langLabel(detailTask.targetLanguage) }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">创建时间</span>
                <span class="detail-value">{{ formatDate(detailTask.createdAt) }}</span>
              </div>
            </div>

            <!-- Transcribe step -->
            <div v-if="detailTask.stepTranscribe" class="detail-section">
              <div class="detail-section__title">
                <span class="step-badge step-badge--done">✓</span>
                Step 2：转录
              </div>
              <div class="detail-row">
                <span class="detail-label">字幕条数</span>
                <span class="detail-value">{{ detailTask.stepTranscribe.subtitleCount }} 条</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">转录提供商</span>
                <span class="detail-value">{{ parseConfigJson(detailTask.stepTranscribe.configJson).providerId ?? '—' }}</span>
              </div>
            </div>

            <!-- Translate step -->
            <div v-if="detailTask.stepTranslate" class="detail-section">
              <div class="detail-section__title">
                <span class="step-badge step-badge--done">✓</span>
                Step 3：翻译
              </div>
              <div class="detail-row">
                <span class="detail-label">字幕条数</span>
                <span class="detail-value">{{ detailTask.stepTranslate.subtitleCount }} 条</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">AI 模型</span>
                <span class="detail-value">{{ parseConfigJson(detailTask.stepTranslate.configJson).aiConfigTitle ?? '—' }}</span>
              </div>
            </div>
          </div>

          <div class="modal__footer">
            <button class="btn btn--primary" @click="continueFromDetail">
              {{ detailTask.status === 'completed' ? '查看工作台' : '继续处理' }}
            </button>
            <button class="btn btn--secondary" @click="closeDetail">关闭</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.projects-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 24px;
  overflow-y: auto;
  gap: 20px;
}

/* ── Header ─────────────────────────────────────────────────────────────── */

.projects-header {
  display: flex;
  align-items: baseline;
  gap: 12px;
  flex-shrink: 0;
}

.projects-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.projects-count {
  font-size: 13px;
  color: var(--text-muted);
}

/* ── Loading / Empty ────────────────────────────────────────────────────── */

.projects-loading {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--text-secondary);
  font-size: 14px;
  padding: 40px 0;
  justify-content: center;
}

.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.projects-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 80px 0;
  color: var(--text-secondary);
}

.projects-empty__icon {
  font-size: 40px;
  margin-bottom: 8px;
}

.projects-empty__text {
  margin: 0;
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
}

.projects-empty__sub {
  margin: 0;
  font-size: 13px;
  color: var(--text-muted);
}

/* ── Task list ──────────────────────────────────────────────────────────── */

.projects-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.task-card {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  cursor: pointer;
  transition: border-color 0.15s ease, background 0.15s ease;
}

.task-card:hover {
  border-color: var(--accent);
  background: var(--bg-hover);
}

.task-card__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.task-card__title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  min-width: 0;
}

.task-card__icon {
  font-size: 16px;
  flex-shrink: 0;
}

.task-card__name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 280px;
}

.task-card__duration {
  font-size: 13px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.task-card__lang {
  font-size: 12px;
  color: var(--text-muted);
  background: var(--bg-base);
  border: 1px solid var(--border);
  padding: 2px 8px;
  border-radius: 20px;
  flex-shrink: 0;
}

.task-card__date {
  font-size: 12px;
  color: var(--text-muted);
  flex-shrink: 0;
  white-space: nowrap;
}

/* ── Step dots ──────────────────────────────────────────────────────────── */

.task-card__progress {
  display: flex;
  align-items: center;
  gap: 12px;
}

.step-dots {
  display: flex;
  gap: 6px;
  align-items: center;
}

.step-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 2px solid var(--dot-color, var(--border));
  background: transparent;
  transition: background 0.15s ease;
  flex-shrink: 0;
}

.step-dot--filled {
  background: var(--dot-color, var(--border));
}

.task-card__status-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.task-card__status-label--done {
  color: var(--status-success);
}

/* ── Actions ────────────────────────────────────────────────────────────── */

.task-card__actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

/* ── Buttons ────────────────────────────────────────────────────────────── */

.btn {
  padding: 7px 18px;
  border-radius: 8px;
  font-size: 13px;
  cursor: pointer;
  border: none;
  transition: all 0.15s ease;
  font-weight: 500;
}

.btn--sm {
  padding: 5px 14px;
  font-size: 12px;
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
  color: var(--text-primary);
}

.btn--danger {
  background: transparent;
  color: var(--status-error);
  border: 1px solid var(--status-error);
}

.btn--danger:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.08);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ── Detail modal ───────────────────────────────────────────────────────── */

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 14px;
  width: 480px;
  max-width: calc(100vw - 48px);
  max-height: calc(100vh - 80px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.modal__title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.modal__close {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 14px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
  transition: background 0.15s;
  flex-shrink: 0;
}

.modal__close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.modal__body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.detail-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.detail-section__title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  padding-bottom: 4px;
  border-bottom: 1px solid var(--border);
}

.step-badge {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  font-size: 10px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.step-badge--done {
  background: var(--status-success);
  color: #fff;
}

.detail-row {
  display: flex;
  gap: 12px;
  font-size: 13px;
}

.detail-label {
  color: var(--text-muted);
  min-width: 80px;
  flex-shrink: 0;
}

.detail-value {
  color: var(--text-primary);
  word-break: break-all;
}

.modal__footer {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}
</style>
