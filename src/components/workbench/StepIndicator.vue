<script setup lang="ts">
import type { StepStatus } from '@/types/workbench'
import { STEP_LABELS } from '@/types/workbench'
import IconUpload from '@/components/icons/IconUpload.vue'
import IconMic from '@/components/icons/IconMic.vue'
import IconLanguage from '@/components/icons/IconLanguage.vue'
import IconVolume from '@/components/icons/IconVolume.vue'
import IconDownload from '@/components/icons/IconDownload.vue'
import IconCheck from '@/components/icons/IconCheck.vue'
import IconLoader from '@/components/icons/IconLoader.vue'
import IconAlertCircle from '@/components/icons/IconAlertCircle.vue'

const STEP_ICONS = [IconUpload, IconMic, IconLanguage, IconVolume, IconDownload]

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
  <div class="step-bar">
    <template v-for="(label, i) in STEP_LABELS" :key="i">
      <div
        class="step-node"
        :class="{
          'step-node--active': i === currentStep,
          'step-node--completed': statuses[i] === 'completed',
          'step-node--processing': statuses[i] === 'processing' && i !== currentStep,
          'step-node--error': statuses[i] === 'error',
          'step-node--idle': statuses[i] === 'idle' && i !== currentStep,
          'step-node--clickable': isClickable(statuses[i]),
        }"
        @click="onStepClick(i)"
      >
        <div class="step-node__circle">
          <IconCheck v-if="statuses[i] === 'completed'" class="step-node__icon" />
          <IconLoader v-else-if="statuses[i] === 'processing'" class="step-node__icon step-node__icon--spin" />
          <IconAlertCircle v-else-if="statuses[i] === 'error'" class="step-node__icon" />
          <component :is="STEP_ICONS[i]" v-else class="step-node__icon" />
        </div>
        <span class="step-node__label">{{ label }}</span>
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
/* ── Bar container ──────────────────────────────────────────────────────── */

.step-bar {
  display: flex;
  align-items: center;
  height: 54px;
  padding: 0 28px;
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  user-select: none;
  gap: 0;
}

/* ── Step node ──────────────────────────────────────────────────────────── */

.step-node {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  cursor: default;
  padding: 5px 8px;
  border-radius: 8px;
  transition: background 0.15s ease;
}

.step-node--clickable {
  cursor: pointer;
}

.step-node--clickable:hover:not(.step-node--active) {
  background: var(--bg-hover);
}

/* ── Circle ─────────────────────────────────────────────────────────────── */

.step-node__circle {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  background: var(--bg-hover);
  color: var(--text-muted);
  border: 1.5px solid var(--border);
  transition: all 0.2s ease;
}

.step-node--active .step-node__circle {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
  box-shadow: 0 0 0 3px var(--accent-subtle);
}

.step-node--processing .step-node__circle {
  background: var(--accent-subtle);
  border-color: var(--accent);
  color: var(--accent);
}

.step-node--completed .step-node__circle {
  background: var(--status-success);
  border-color: var(--status-success);
  color: #fff;
}

.step-node--error .step-node__circle {
  background: var(--status-error-subtle);
  border-color: var(--status-error);
  color: var(--status-error);
}

.step-node--idle {
  opacity: 0.4;
}

.step-node__icon {
  width: 13px;
  height: 13px;
}

.step-node__icon--spin {
  animation: spin 1.2s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}

/* ── Label ──────────────────────────────────────────────────────────────── */

.step-node__label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-muted);
  white-space: nowrap;
  transition: color 0.15s ease;
}

.step-node--active .step-node__label {
  color: var(--text-primary);
  font-weight: 600;
}

.step-node--completed .step-node__label {
  color: var(--status-success);
}

.step-node--error .step-node__label {
  color: var(--status-error);
}

/* ── Connector ──────────────────────────────────────────────────────────── */

.step-connector {
  flex: 1;
  height: 1.5px;
  background: var(--border);
  min-width: 12px;
  border-radius: 1px;
  transition: background 0.3s ease;
}

.step-connector--done {
  background: var(--status-success);
}

/* ── Responsive ─────────────────────────────────────────────────────────── */

@container workbench-root (max-width: 760px) {
  .step-node__label { display: none; }
  .step-node { padding: 5px 4px; }
}

@media (max-height: 540px) {
  .step-bar { height: 44px; }
}
</style>
