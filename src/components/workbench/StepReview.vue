<script setup lang="ts">
import { useWorkbench } from '@/composables/useWorkbench'
import SubtitleEditor from './SubtitleEditor.vue'

const {
  originalSubtitles, translatedSubtitles, setStepStatus,
} = useWorkbench()

function onUpdateTranslation(id: number, text: string) {
  const sub = translatedSubtitles.value.find(s => s.id === id)
  if (sub) sub.text = text
}

function confirmSubtitles() {
  setStepStatus(3, 'completed')
}
</script>

<template>
  <div class="step-review">
    <SubtitleEditor
      :original="originalSubtitles"
      :translated="translatedSubtitles"
      @update-translation="onUpdateTranslation"
    />
    <div class="step-review__actions">
      <button class="btn btn--primary" @click="confirmSubtitles">确认字幕</button>
    </div>
  </div>
</template>

<style scoped>
.step-review {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}

.step-review__actions {
  display: flex;
  justify-content: flex-end;
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
</style>
