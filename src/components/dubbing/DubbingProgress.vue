<script setup lang="ts">
import { computed, ref } from 'vue'
import type { DubbingStage, DubbingStatus, TtsItemProgress } from '@/types/dubbing'
import { DUBBING_STAGE_LABELS } from '@/types/dubbing'
import ProgressBar from '@/components/workbench/ProgressBar.vue'
import SubtitleRow from '@/components/workbench/SubtitleRow.vue'

const props = defineProps<{
  stageStatuses: Record<DubbingStage, DubbingStatus>
  stageProgress: Record<DubbingStage, number>
  currentMessage?: string
  originalSubtitles?: Array<{ id: number; text: string; startTime: number; endTime: number }>
  livePreprocessed?: Map<number, string>
  ttsTotal?: number
  ttsCompleted?: number
  ttsItems?: Map<number, TtsItemProgress>
}>()

const emit = defineEmits<{ cancel: [] }>()

const STAGE_ORDER: DubbingStage[] = [
  'preprocess', 'media', 'reference', 'tts', 'alignment', 'compose',
]

// Descriptions shown in the log card for simple stages
const STAGE_DESCRIPTIONS: Record<DubbingStage, string> = {
  preprocess: '通过 AI 将字幕文本规范化，生成适合 TTS 朗读的口语化文本',
  media: '使用 FFmpeg 分离视频音轨与静音视频',
  reference: '提取或准备参考音频片段',
  tts: '逐条调用 TTS 引擎生成语音文件',
  alignment: '将 TTS 音频与字幕时间轴对齐并拼接',
  compose: '将配音音频与静音视频合成最终输出',
}

// The first stage that is currently running (or the last completed if all done)
const activeStage = computed<DubbingStage>(() => {
  const running = STAGE_ORDER.find(s => props.stageStatuses[s] === 'running')
  if (running) return running
  // Fall back to last completed stage
  const completed = [...STAGE_ORDER].reverse().find(s => props.stageStatuses[s] === 'completed')
  return completed ?? 'preprocess'
})

const activePercent = computed(() => props.stageProgress[activeStage.value] ?? 0)

// Preprocess completed count = size of livePreprocessed map
const preprocessDoneCount = computed(() => props.livePreprocessed?.size ?? 0)
const preprocessTotal = computed(() => props.originalSubtitles?.length ?? 0)

// Preprocess subtitle rows for dual-column display
const preprocessRows = computed(() =>
  (props.originalSubtitles ?? []).map(sub => ({
    ...sub,
    preprocessedText: props.livePreprocessed?.get(sub.id) ?? null,
  }))
)

function pillStatus(stage: DubbingStage): 'pending' | 'running' | 'completed' | 'failed' {
  return props.stageStatuses[stage]
}

function ttsItemIcon(idx: number): string {
  const s = props.ttsItems?.get(idx)?.status
  if (s === 'completed') return '✓'
  if (s === 'failed') return '✕'
  return '…'
}

// Scroll sync for preprocess dual-column
const leftCol = ref<HTMLElement>()
const rightCol = ref<HTMLElement>()
let syncing = false

function syncScroll(source: HTMLElement, target: HTMLElement) {
  if (syncing) return
  syncing = true
  target.scrollTop = source.scrollTop
  requestAnimationFrame(() => { syncing = false })
}
function onLeftScroll() {
  if (leftCol.value && rightCol.value) syncScroll(leftCol.value, rightCol.value)
}
function onRightScroll() {
  if (rightCol.value && leftCol.value) syncScroll(rightCol.value, leftCol.value)
}
</script>

<template>
  <div class="dubbing-panel">
    <!-- Phase bar: stage pills + cancel -->
    <div class="phase-bar">
      <div class="phase-pills">
        <div
          v-for="(stage, i) in STAGE_ORDER"
          :key="stage"
          class="phase-pill"
          :class="`phase-pill--${pillStatus(stage)}`"
        >
          <span v-if="pillStatus(stage) === 'running'" class="phase-pill__spinner"></span>
          <span v-else-if="pillStatus(stage) === 'completed'" class="phase-pill__check">✓</span>
          <span v-else-if="pillStatus(stage) === 'failed'" class="phase-pill__x">✕</span>
          <span class="phase-pill__num">{{ i + 1 }}</span>
          {{ DUBBING_STAGE_LABELS[stage] }}
        </div>
      </div>
      <button class="btn btn--secondary btn--sm" @click="emit('cancel')">取消</button>
    </div>

    <!-- Progress row -->
    <div class="progress-row">
      <span class="progress-row__msg">{{ currentMessage || DUBBING_STAGE_LABELS[activeStage] }}</span>
      <div class="progress-row__bar">
        <ProgressBar :percent="activePercent" show-percent />
      </div>
      <span v-if="activeStage === 'tts' && ttsTotal" class="progress-row__count">
        {{ ttsCompleted }}/{{ ttsTotal }}
      </span>
    </div>

    <!-- Detail area -->
    <div class="detail-area">

      <!-- Preprocess: dual-column subtitle list -->
      <template v-if="activeStage === 'preprocess' && originalSubtitles?.length">
        <div class="dual-col-header">
          <span>原文 ({{ preprocessTotal }} 条)</span>
          <span>预处理 (<span class="accent">{{ preprocessDoneCount }}</span>/{{ preprocessTotal }} 完成)</span>
        </div>
        <div class="dual-col-body">
          <div ref="leftCol" class="dual-col" @scroll="onLeftScroll">
            <SubtitleRow
              v-for="sub in originalSubtitles"
              :key="sub.id"
              :subtitle="sub"
            />
          </div>
          <div ref="rightCol" class="dual-col" @scroll="onRightScroll">
            <SubtitleRow
              v-for="row in preprocessRows"
              :key="row.id"
              :subtitle="{ ...row, text: row.preprocessedText ?? '' }"
              :loading="row.preprocessedText === null"
            />
          </div>
        </div>
      </template>

      <!-- TTS: per-item list -->
      <template v-else-if="activeStage === 'tts' && originalSubtitles?.length">
        <div class="tts-list">
          <div
            v-for="(sub, i) in originalSubtitles"
            :key="sub.id"
            class="tts-item"
            :class="`tts-item--${ttsItems?.get(i)?.status ?? 'pending'}`"
          >
            <span class="tts-item__index">{{ i + 1 }}</span>
            <span class="tts-item__icon">{{ ttsItemIcon(i) }}</span>
            <span class="tts-item__text">{{ sub.text.length > 60 ? sub.text.slice(0, 60) + '…' : sub.text }}</span>
          </div>
        </div>
      </template>

      <!-- Log card for other stages -->
      <template v-else>
        <div class="log-card">
          <div class="log-card__spinner"></div>
          <p class="log-card__stage">{{ DUBBING_STAGE_LABELS[activeStage] }}</p>
          <p class="log-card__desc">{{ STAGE_DESCRIPTIONS[activeStage] }}</p>
          <p v-if="currentMessage" class="log-card__msg">{{ currentMessage }}</p>
        </div>
      </template>

    </div>
  </div>
</template>

<style scoped>
.dubbing-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: 10px;
}

/* ── Phase bar ────────────────────────────────────────────────────────── */

.phase-bar {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elevated);
  flex-shrink: 0;
}

.phase-pills {
  display: flex;
  gap: 6px;
  flex: 1;
  flex-wrap: wrap;
}

.phase-pill {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s ease;
  border: 1px solid var(--border);
  color: var(--text-muted);
  background: var(--bg-base);
}

.phase-pill--running {
  background: var(--accent);
  color: #fff;
  border-color: transparent;
}

.phase-pill--completed {
  background: transparent;
  color: var(--status-success);
  border-color: var(--status-success);
}

.phase-pill--failed {
  background: transparent;
  color: var(--status-error);
  border-color: var(--status-error);
}

.phase-pill__num {
  opacity: 0.7;
  font-size: 10px;
}

.phase-pill__spinner {
  width: 10px;
  height: 10px;
  border: 2px solid rgba(255, 255, 255, 0.4);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}

.phase-pill__check {
  font-size: 10px;
  font-weight: 700;
}

.phase-pill__x {
  font-size: 10px;
  font-weight: 700;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* ── Progress row ─────────────────────────────────────────────────────── */

.progress-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.progress-row__msg {
  font-size: 13px;
  color: var(--text-secondary);
  min-width: 120px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.progress-row__bar {
  flex: 1;
  min-width: 0;
}

.progress-row__count {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
  flex-shrink: 0;
}

/* ── Detail area ──────────────────────────────────────────────────────── */

.detail-area {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ── Dual-col (preprocess) ────────────────────────────────────────────── */

.dual-col-header {
  display: grid;
  grid-template-columns: 1fr 1fr;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elevated);
  flex-shrink: 0;
}

.dual-col-header span {
  padding: 8px 16px;
  font-size: 13px;
  color: var(--text-secondary);
}

.dual-col-header span:first-child {
  border-right: 1px solid var(--border);
}

.accent {
  color: var(--accent);
  font-weight: 500;
}

.dual-col-body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.dual-col {
  overflow-y: auto;
  min-height: 0;
  scrollbar-width: thin;
  scrollbar-color: var(--border) transparent;
}

.dual-col:first-child {
  border-right: 1px solid var(--border);
}

/* ── TTS item list ────────────────────────────────────────────────────── */

.tts-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px;
  scrollbar-width: thin;
  scrollbar-color: var(--border) transparent;
}

.tts-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-radius: 6px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  transition: border-color 0.15s;
  flex-shrink: 0;
}

.tts-item--completed {
  border-color: var(--status-success);
}

.tts-item--failed {
  border-color: var(--status-error);
}

.tts-item__index {
  font-size: 11px;
  color: var(--text-muted);
  width: 28px;
  text-align: right;
  flex-shrink: 0;
}

.tts-item__icon {
  font-size: 12px;
  width: 16px;
  text-align: center;
  flex-shrink: 0;
  color: var(--text-muted);
}

.tts-item--completed .tts-item__icon { color: var(--status-success); }
.tts-item--failed .tts-item__icon { color: var(--status-error); }

.tts-item__text {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ── Log card ─────────────────────────────────────────────────────────── */

.log-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 32px;
}

.log-card__spinner {
  width: 36px;
  height: 36px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.9s linear infinite;
  flex-shrink: 0;
}

.log-card__stage {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.log-card__desc {
  margin: 0;
  font-size: 13px;
  color: var(--text-muted);
  text-align: center;
  max-width: 380px;
}

.log-card__msg {
  margin: 0;
  font-size: 13px;
  color: var(--text-secondary);
  padding: 8px 16px;
  background: var(--bg-elevated);
  border-radius: 6px;
  border: 1px solid var(--border);
}

/* ── Buttons ──────────────────────────────────────────────────────────── */

.btn {
  padding: 8px 20px;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  border: none;
  transition: all 0.15s ease;
}

.btn--secondary {
  background: var(--bg-hover);
  color: var(--text-secondary);
  border: 1px solid var(--border);
}

.btn--secondary:hover {
  background: var(--bg-elevated);
}

.btn--sm {
  padding: 5px 14px;
  font-size: 13px;
}
</style>
