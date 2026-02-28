<script setup lang="ts">
import { ref, watch } from 'vue'
import { useWorkbench } from '@/composables/useWorkbench'
import StepIndicator from '@/components/workbench/StepIndicator.vue'
import StepUpload from '@/components/workbench/StepUpload.vue'
import StepTranscribe from '@/components/workbench/StepTranscribe.vue'
import StepTranslate from '@/components/workbench/StepTranslate.vue'
import StepReview from '@/components/workbench/StepReview.vue'
import StepDubbing from '@/components/workbench/StepDubbing.vue'
import StepExport from '@/components/workbench/StepExport.vue'

const {
  currentStep, stepStatuses, goToStep, goNext, goPrev,
  canGoNext, canGoPrev,
} = useWorkbench()

const direction = ref<'forward' | 'backward'>('forward')

watch(currentStep, (newVal, oldVal) => {
  direction.value = newVal > oldVal ? 'forward' : 'backward'
})

function handleNext() {
  direction.value = 'forward'
  goNext()
}

function handlePrev() {
  direction.value = 'backward'
  goPrev()
}
</script>

<template>
  <div class="workbench">
    <div class="workbench__body">
      <StepIndicator
        :current-step="currentStep"
        :statuses="stepStatuses"
        @navigate="goToStep"
      />

      <div class="workbench__content">
        <Transition :name="direction === 'forward' ? 'step-fwd' : 'step-bwd'" mode="out-in">
          <StepUpload v-if="currentStep === 0" key="0" />
          <StepTranscribe v-else-if="currentStep === 1" key="1" />
          <StepTranslate v-else-if="currentStep === 2" key="2" />
          <StepReview v-else-if="currentStep === 3" key="3" />
          <StepDubbing v-else-if="currentStep === 4" key="4" />
          <StepExport v-else-if="currentStep === 5" key="5" />
        </Transition>
      </div>
    </div>

    <div class="workbench__footer">
      <button
        class="btn btn--secondary"
        :disabled="!canGoPrev"
        @click="handlePrev"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6" />
        </svg>
        上一步
      </button>
      <button
        class="btn btn--primary"
        :disabled="!canGoNext"
        @click="handleNext"
      >
        下一步
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="9 18 15 12 9 6" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.workbench {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.workbench__body {
  display: flex;
  flex: 1;
  min-height: 0;
}

.workbench__content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 24px;
  position: relative;
}

.workbench__footer {
  display: flex;
  justify-content: space-between;
  padding: 14px 24px;
  border-top: 1px solid var(--border);
}

/* ── Buttons ─────────────────────────────────────────────────────────── */

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 22px;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: all 0.18s ease;
  letter-spacing: 0.01em;
}

.btn--primary {
  background: var(--accent);
  color: #fff;
  box-shadow: 0 2px 8px rgba(99, 102, 241, 0.35);
}

.btn--primary:hover:not(:disabled) {
  background: var(--accent-hover);
  box-shadow: 0 4px 14px rgba(99, 102, 241, 0.45);
  transform: translateY(-1px);
}

.btn--primary:active:not(:disabled) {
  transform: translateY(0);
  box-shadow: 0 2px 6px rgba(99, 102, 241, 0.30);
}

.btn--secondary {
  background: var(--bg-elevated);
  color: var(--text-secondary);
  border: 1px solid var(--border);
}

.btn--secondary:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--text-muted);
}

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* ── Step transition: forward (slide left) ────────────────────────────── */

.step-fwd-enter-active,
.step-fwd-leave-active,
.step-bwd-enter-active,
.step-bwd-leave-active {
  transition: opacity 0.24s cubic-bezier(0.4, 0, 0.2, 1),
              transform 0.24s cubic-bezier(0.4, 0, 0.2, 1);
}

.step-fwd-enter-from {
  opacity: 0;
  transform: translateX(28px);
}

.step-fwd-leave-to {
  opacity: 0;
  transform: translateX(-28px);
}

.step-bwd-enter-from {
  opacity: 0;
  transform: translateX(-28px);
}

.step-bwd-leave-to {
  opacity: 0;
  transform: translateX(28px);
}
</style>
