<script setup lang="ts">
import type { StepStatus } from '@/types/workbench'
import { STEP_LABELS } from '@/types/workbench'
import IconUpload from '@/components/icons/IconUpload.vue'
import IconMic from '@/components/icons/IconMic.vue'
import IconLanguage from '@/components/icons/IconLanguage.vue'
import IconCheck from '@/components/icons/IconCheck.vue'
import IconVolume from '@/components/icons/IconVolume.vue'
import IconDownload from '@/components/icons/IconDownload.vue'
import IconLoader from '@/components/icons/IconLoader.vue'
import IconAlertCircle from '@/components/icons/IconAlertCircle.vue'

const STEP_ICONS = [IconUpload, IconMic, IconLanguage, IconCheck, IconVolume, IconDownload]

const STATUS_LABELS: Record<StepStatus, string> = {
  idle: '等待中',
  ready: '已就绪',
  processing: '进行中',
  completed: '已完成',
  error: '错误',
}

const props = defineProps<{
  currentStep: number
  statuses: StepStatus[]
}>()

const emit = defineEmits<{
  navigate: [step: number]
}>()

function isClickable(status: StepStatus): boolean {
  return status === 'completed' || status === 'ready'
}

function onStepClick(index: number) {
  if (isClickable(props.statuses[index])) {
    emit('navigate', index)
  }
}
</script>

<template>
  <div class="step-panel">
    <template v-for="(label, i) in STEP_LABELS" :key="i">
      <div
        class="step-item"
        :class="{
          'step-item--active': i === currentStep,
          'step-item--completed': statuses[i] === 'completed',
          'step-item--processing': statuses[i] === 'processing',
          'step-item--error': statuses[i] === 'error',
          'step-item--ready': statuses[i] === 'ready' && i !== currentStep,
          'step-item--idle': statuses[i] === 'idle' && i !== currentStep,
          'step-item--clickable': isClickable(statuses[i]),
        }"
        @click="onStepClick(i)"
      >
        <div class="step-item__icon-wrap">
          <IconCheck v-if="statuses[i] === 'completed'" class="step-item__icon" />
          <IconLoader v-else-if="statuses[i] === 'processing'" class="step-item__icon step-item__icon--spin" />
          <IconAlertCircle v-else-if="statuses[i] === 'error'" class="step-item__icon" />
          <component :is="STEP_ICONS[i]" v-else class="step-item__icon" />
        </div>

        <div class="step-item__text">
          <span class="step-item__name">{{ label }}</span>
          <span class="step-item__status">{{ STATUS_LABELS[statuses[i]] }}</span>
        </div>
      </div>

      <div
        v-if="i < STEP_LABELS.length - 1"
        class="step-connector"
        :class="{ 'step-connector--done': statuses[i] === 'completed' }"
      />
    </template>
  </div>
</template>

<style scoped>
.step-panel {
  width: 176px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  background: var(--bg-elevated);
  overflow-y: auto;
  overflow-x: hidden;
  user-select: none;
  padding: 20px 0;
}

/* ── Step item ─────────────────────────────────────────────────────────── */

.step-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 16px;
  height: 56px;
  flex-shrink: 0;
  border-left: 3px solid transparent;
  transition: background 0.15s ease, border-color 0.15s ease;
}

.step-item--active {
  border-left-color: var(--accent);
  background: var(--accent-subtle);
}

.step-item--processing:not(.step-item--active) {
  border-left-color: var(--accent);
  background: var(--accent-subtle);
}

.step-item--error {
  border-left-color: var(--status-error);
}

.step-item--idle {
  opacity: 0.5;
}

.step-item--clickable {
  cursor: pointer;
}

.step-item--clickable:hover:not(.step-item--active) {
  background: var(--bg-hover);
}

/* ── Icon circle ───────────────────────────────────────────────────────── */

.step-item__icon-wrap {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  background: var(--bg-hover);
  color: var(--text-muted);
  transition: background 0.18s ease, color 0.18s ease;
}

.step-item--active .step-item__icon-wrap {
  background: var(--accent-subtle);
  color: var(--accent);
}

.step-item--processing .step-item__icon-wrap {
  background: var(--accent-subtle);
  color: var(--accent);
}

.step-item--completed .step-item__icon-wrap {
  background: var(--status-success-subtle);
  color: var(--status-success);
}

.step-item--error .step-item__icon-wrap {
  background: var(--status-error-subtle);
  color: var(--status-error);
}

.step-item__icon {
  width: 14px;
  height: 14px;
}

.step-item__icon--spin {
  animation: spin 1.2s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}

/* ── Text block ────────────────────────────────────────────────────────── */

.step-item__text {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.step-item__name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  line-height: 1.2;
  transition: color 0.15s ease, font-weight 0.15s ease;
}

.step-item--active .step-item__name {
  color: var(--text-primary);
  font-weight: 600;
}

.step-item__status {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1;
}

.step-item--processing .step-item__status {
  color: var(--accent);
  animation: pulse-text 1.5s ease-in-out infinite;
}

.step-item--completed .step-item__status {
  color: var(--status-success);
}

.step-item--error .step-item__status {
  color: var(--status-error);
}

@keyframes pulse-text {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.45; }
}

/* ── Connector line ────────────────────────────────────────────────────── */

.step-connector {
  /* 16px padding-left + 14px (half of 28px icon) - 1px (half of 2px line) = 29px */
  margin-left: 29px;
  width: 2px;
  height: 16px;
  background: var(--border);
  flex-shrink: 0;
  border-radius: 1px;
  transition: background 0.2s ease;
}

.step-connector--done {
  background: var(--status-success);
}
</style>
