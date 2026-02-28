<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { useWorkbench } from '@/composables/useWorkbench'
import { useTranslationSettings } from '@/composables/useTranslationSettings'
import { useAiConfigs } from '@/composables/useAiConfigs'
import { LANGUAGES, TARGET_LANGUAGES } from '@/types/workbench'
import ProgressBar from './ProgressBar.vue'
import SubtitleRow from './SubtitleRow.vue'
import SubtitleEditor from './SubtitleEditor.vue'

const {
  targetLanguage, sourceLanguage, stepStatuses, setStepStatus,
  originalSubtitles, translatedSubtitles, progress, projectDir,
} = useWorkbench()

const { translationSettings } = useTranslationSettings()
const { aiConfigs } = useAiConfigs()

const defaultConfig = computed(() => aiConfigs.value.find(c => c.isDefault))
const errorMsg = ref('')

let unlisten: UnlistenFn | null = null
let unlistenBatch: UnlistenFn | null = null

// ── Local translation progress state ───────────────────────────────────────

interface TranslateProgressPayload {
  phase: string
  batch: number
  totalBatches: number
  skipped: number
  percent: number
  message: string
}

const translateProgress = ref<TranslateProgressPayload>({
  phase: '',
  batch: 0,
  totalBatches: 0,
  skipped: 0,
  percent: 0,
  message: '',
})

// Live translations: subtitle id → translated text
const liveTranslations = ref<Map<number, string>>(new Map())

// ── Phase list ──────────────────────────────────────────────────────────────

const phaseList = computed(() => {
  const ts = translationSettings.value
  const phases: Array<{ key: string; label: string; color: string }> = []
  if (ts.correction) phases.push({ key: 'correction', label: '校正', color: '#f59e0b' })
  phases.push({ key: 'translation', label: '翻译', color: 'var(--accent)' })
  if (ts.optimization) phases.push({ key: 'optimization', label: '优化', color: '#8b5cf6' })
  return phases
})

function phaseStatus(label: string): 'idle' | 'active' | 'completed' {
  const activePhase = translateProgress.value.phase
  const activeIdx = phaseList.value.findIndex(p => p.label === activePhase)
  const thisIdx = phaseList.value.findIndex(p => p.label === label)
  if (activeIdx === -1) return 'idle'
  if (thisIdx < activeIdx) return 'completed'
  if (thisIdx === activeIdx) return 'active'
  return 'idle'
}

// ── Computed subtitle rows for right column ─────────────────────────────────

const translationRows = computed(() =>
  originalSubtitles.value.map(sub => ({
    ...sub,
    text: liveTranslations.value.get(sub.id) ?? '',
  }))
)

const translatedCount = computed(() => liveTranslations.value.size)

// ── Scroll sync ─────────────────────────────────────────────────────────────

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

// ── Translation control ─────────────────────────────────────────────────────

async function startTranslation() {
  if (!defaultConfig.value) {
    errorMsg.value = '请先在设置中添加并设为默认 AI 配置'
    return
  }
  errorMsg.value = ''
  liveTranslations.value = new Map()
  translateProgress.value = { phase: '', batch: 0, totalBatches: 0, skipped: 0, percent: 0, message: '' }
  setStepStatus(2, 'processing')

  unlisten = await listen<TranslateProgressPayload>('translate:progress', (event) => {
    translateProgress.value = event.payload
  })

  unlistenBatch = await listen<{ phase: string; updates: Array<{ index: number; text: string }> }>('translate:batch_result', (event) => {
    const newMap = new Map(liveTranslations.value)
    for (const { index, text } of event.payload.updates) {
      const sub = originalSubtitles.value[index]
      if (sub) newMap.set(sub.id, text)
    }
    liveTranslations.value = newMap
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
      promptCorrection: ts.promptCorrection,
      promptStandard: ts.promptStandard,
      promptReflective: ts.promptReflective,
      promptOptimize: ts.promptOptimize,
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
    unlistenBatch?.()
    unlistenBatch = null
  }
}

async function cancelTranslation() {
  await invoke('cmd_cancel_translation')
}

function resetTranslation() {
  setStepStatus(2, 'ready')
  translatedSubtitles.value = []
  liveTranslations.value = new Map()
  translateProgress.value = { phase: '', batch: 0, totalBatches: 0, skipped: 0, percent: 0, message: '' }
}

function onUpdateTranslation(id: number, text: string) {
  const sub = translatedSubtitles.value.find(s => s.id === id)
  if (sub) sub.text = text
}

onUnmounted(() => {
  unlisten?.()
  unlistenBatch?.()
})
</script>

<template>
  <div class="step-translate">
    <!-- Config state -->
    <div v-if="stepStatuses[2] === 'ready'" class="step-translate__center">
      <div class="config-panel">
        <div class="field">
          <label class="field-label">目标语言</label>
          <select v-model="targetLanguage" class="select">
            <option v-for="l in TARGET_LANGUAGES" :key="l.code" :value="l.code">{{ l.label }}</option>
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
    </div>

    <!-- Processing state — full-width -->
    <div v-else-if="stepStatuses[2] === 'processing'" class="translate-processing">
      <!-- Phase pills + cancel -->
      <div class="phase-bar">
        <div class="phase-pills">
          <div
            v-for="p in phaseList"
            :key="p.key"
            class="phase-pill"
            :class="`phase-pill--${phaseStatus(p.label)}`"
            :style="phaseStatus(p.label) !== 'idle' ? `--phase-color: ${p.color}` : ''"
          >
            <span v-if="phaseStatus(p.label) === 'active'" class="phase-pill__spinner"></span>
            <span v-else-if="phaseStatus(p.label) === 'completed'" class="phase-pill__check">✓</span>
            {{ p.label }}
          </div>
        </div>
        <button class="btn btn--secondary btn--sm" @click="cancelTranslation">取消</button>
      </div>

      <!-- Progress bar row -->
      <div class="translate-progress-row">
        <span class="translate-progress-row__msg">{{ translateProgress.message }}</span>
        <div class="translate-progress-bar-wrap">
          <ProgressBar :percent="translateProgress.percent" show-percent />
        </div>
        <span v-if="translateProgress.skipped > 0" class="translate-progress-row__skipped">
          已跳过 {{ translateProgress.skipped }} 条
        </span>
      </div>

      <!-- Dual column subtitle list -->
      <div class="translate-subtitle-area">
        <div class="translate-subtitle-header">
          <span>原文 ({{ originalSubtitles.length }} 条)</span>
          <span>翻译中 (<span class="translate-subtitle-count">{{ translatedCount }}</span>/{{ originalSubtitles.length }} 完成)</span>
        </div>
        <div class="translate-subtitle-body">
          <div ref="leftCol" class="translate-subtitle-col" @scroll="onLeftScroll">
            <SubtitleRow
              v-for="sub in originalSubtitles"
              :key="sub.id"
              :subtitle="sub"
            />
          </div>
          <div ref="rightCol" class="translate-subtitle-col" @scroll="onRightScroll">
            <SubtitleRow
              v-for="(sub, idx) in translationRows"
              :key="sub.id"
              :subtitle="sub"
              :loading="!liveTranslations.has(originalSubtitles[idx]?.id)"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Completed state — full-width card with SubtitleEditor -->
    <div v-else-if="stepStatuses[2] === 'completed'" class="translate-completed">
      <div class="translate-header">
        <div class="translate-header__badge">
          <span class="badge-icon">✓</span>
          <span>翻译完成</span>
        </div>
        <div class="translate-header__meta">
          <span>{{ originalSubtitles.length }} 条字幕</span>
          <span>{{ sourceLanguage }} → {{ targetLanguage }}</span>
        </div>
        <button class="btn btn--ghost" @click="resetTranslation">重新翻译</button>
      </div>
      <div class="translate-editor-body">
        <SubtitleEditor
          :original="originalSubtitles"
          :translated="translatedSubtitles"
          @updateTranslation="onUpdateTranslation"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.step-translate {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.step-translate__center {
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

/* ── Processing state ──────────────────────────────────────────────────── */

.translate-processing {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: 10px;
}

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
  gap: 8px;
  flex: 1;
  flex-wrap: wrap;
}

.phase-pill {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 12px;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.phase-pill--idle {
  background: var(--bg-base);
  color: var(--text-muted);
  border: 1px solid var(--border);
}

.phase-pill--active {
  background: var(--phase-color);
  color: #fff;
  border: 1px solid transparent;
}

.phase-pill--completed {
  background: transparent;
  color: var(--phase-color);
  border: 1px solid var(--phase-color);
}

.phase-pill__spinner {
  width: 11px;
  height: 11px;
  border: 2px solid rgba(255, 255, 255, 0.4);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}

.phase-pill__check {
  font-size: 11px;
  font-weight: 700;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.translate-progress-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.translate-progress-row__msg {
  font-size: 13px;
  color: var(--text-secondary);
  min-width: 100px;
  white-space: nowrap;
}

.translate-progress-bar-wrap {
  flex: 1;
  min-width: 0;
}

.translate-progress-row__skipped {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
  flex-shrink: 0;
}

.translate-subtitle-area {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.translate-subtitle-header {
  display: grid;
  grid-template-columns: 1fr 1fr;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elevated);
  flex-shrink: 0;
}

.translate-subtitle-header span {
  padding: 8px 16px;
  font-size: 13px;
  color: var(--text-secondary);
}

.translate-subtitle-header span:first-child {
  border-right: 1px solid var(--border);
}

.translate-subtitle-count {
  color: var(--accent);
  font-weight: 500;
}

.translate-subtitle-body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.translate-subtitle-col {
  overflow-y: auto;
  min-height: 0;
}

.translate-subtitle-col:first-child {
  border-right: 1px solid var(--border);
}

/* ── Completed state ───────────────────────────────────────────────────── */

.translate-completed {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: 10px;
}

.translate-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elevated);
  flex-shrink: 0;
}

.translate-header__badge {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  font-weight: 500;
  color: var(--status-success);
}

.translate-header__meta {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--text-secondary);
  flex: 1;
}

.translate-editor-body {
  flex: 1;
  min-height: 0;
  padding: 0;
  overflow: hidden;
}

/* ── Shared ────────────────────────────────────────────────────────────── */

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

.btn--primary:hover:not(:disabled) {
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

.btn--sm {
  padding: 5px 14px;
  font-size: 13px;
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

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
