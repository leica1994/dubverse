<script setup lang="ts">
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
</script>

<template>
  <div class="workbench">
    <StepIndicator
      :current-step="currentStep"
      :statuses="stepStatuses"
      @navigate="goToStep"
    />

    <div class="workbench__content">
      <StepUpload v-if="currentStep === 0" />
      <StepTranscribe v-else-if="currentStep === 1" />
      <StepTranslate v-else-if="currentStep === 2" />
      <StepReview v-else-if="currentStep === 3" />
      <StepDubbing v-else-if="currentStep === 4" />
      <StepExport v-else-if="currentStep === 5" />
    </div>

    <div class="workbench__footer">
      <button
        class="btn btn--secondary"
        :disabled="!canGoPrev"
        @click="goPrev"
      >← 上一步</button>
      <button
        class="btn btn--primary"
        :disabled="!canGoNext"
        @click="goNext"
      >下一步 →</button>
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

.workbench__content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 8px 0;
}

.workbench__footer {
  display: flex;
  justify-content: space-between;
  padding: 16px 0 0;
  border-top: 1px solid var(--border);
  margin-top: auto;
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
  background: var(--bg-elevated);
  color: var(--text-secondary);
  border: 1px solid var(--border);
}

.btn--secondary:hover {
  background: var(--bg-hover);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
