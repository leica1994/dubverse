<script setup lang="ts">
import type { StepStatus } from '@/types/workbench'
import { STEP_LABELS } from '@/types/workbench'
import IconCheck from '@/components/icons/IconCheck.vue'
import IconLoader from '@/components/icons/IconLoader.vue'
import IconAlertCircle from '@/components/icons/IconAlertCircle.vue'

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
  <div class="step-indicator">
    <div
      v-for="(label, i) in STEP_LABELS"
      :key="i"
      class="step-indicator__item"
      :class="{ 'step-indicator__item--clickable': isClickable(statuses[i]) }"
      @click="onStepClick(i)"
    >
      <div
        v-if="i > 0"
        class="step-indicator__line"
        :class="{ 'step-indicator__line--done': statuses[i - 1] === 'completed' }"
      />
      <div
        class="step-indicator__node"
        :class="{
          'step-indicator__node--active': i === currentStep,
          'step-indicator__node--completed': statuses[i] === 'completed',
          'step-indicator__node--processing': statuses[i] === 'processing',
          'step-indicator__node--error': statuses[i] === 'error',
          'step-indicator__node--idle': statuses[i] === 'idle',
        }"
      >
        <IconCheck v-if="statuses[i] === 'completed'" class="step-indicator__icon" />
        <IconLoader v-else-if="statuses[i] === 'processing'" class="step-indicator__icon" />
        <IconAlertCircle v-else-if="statuses[i] === 'error'" class="step-indicator__icon" />
        <span v-else class="step-indicator__num">{{ i + 1 }}</span>
      </div>
      <span
        class="step-indicator__label"
        :class="{ 'step-indicator__label--active': i === currentStep }"
      >{{ label }}</span>
    </div>
  </div>
</template>

<style scoped>
.step-indicator {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 16px 0 20px;
  user-select: none;
}

.step-indicator__item {
  display: flex;
  flex-direction: column;
  align-items: center;
  position: relative;
  flex: 1;
  max-width: 160px;
}

.step-indicator__item--clickable {
  cursor: pointer;
}

.step-indicator__line {
  position: absolute;
  top: calc(var(--step-node-size) / 2);
  right: 50%;
  width: 100%;
  height: 2px;
  background: var(--border);
  z-index: 0;
  transform: translateX(-50%);
}

.step-indicator__line--done {
  background: var(--status-success);
}

.step-indicator__node {
  width: var(--step-node-size);
  height: var(--step-node-size);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px solid var(--border);
  background: var(--bg-elevated);
  position: relative;
  z-index: 1;
  transition: all 0.15s ease;
}

.step-indicator__node--active {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-subtle);
}

.step-indicator__node--completed {
  border-color: var(--status-success);
  background: var(--status-success);
  color: #fff;
}

.step-indicator__node--processing {
  border-color: var(--accent);
  color: var(--accent);
}

.step-indicator__node--error {
  border-color: var(--status-error);
  color: var(--status-error);
}

.step-indicator__node--idle {
  opacity: 0.5;
}

.step-indicator__icon {
  width: 16px;
  height: 16px;
}

.step-indicator__num {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.step-indicator__node--active .step-indicator__num {
  color: var(--accent);
}

.step-indicator__label {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
}

.step-indicator__label--active {
  color: var(--text-primary);
  font-weight: 500;
}
</style>
