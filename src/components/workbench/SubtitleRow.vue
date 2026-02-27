<script setup lang="ts">
import type { Subtitle } from '@/types/workbench'

defineProps<{
  subtitle: Subtitle
  editable?: boolean
}>()

const emit = defineEmits<{
  update: [text: string]
}>()

function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  const ms = Math.floor((seconds % 1) * 100)
  return `${m}:${s.toString().padStart(2, '0')}.${ms.toString().padStart(2, '0')}`
}

function onInput(e: Event) {
  emit('update', (e.target as HTMLTextAreaElement).value)
}
</script>

<template>
  <div class="subtitle-row">
    <span class="subtitle-row__time">
      {{ formatTime(subtitle.startTime) }} - {{ formatTime(subtitle.endTime) }}
    </span>
    <textarea
      v-if="editable"
      class="subtitle-row__text subtitle-row__text--editable"
      :value="subtitle.text"
      rows="2"
      @input="onInput"
    />
    <p v-else class="subtitle-row__text">{{ subtitle.text }}</p>
  </div>
</template>

<style scoped>
.subtitle-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
}

.subtitle-row__time {
  font-size: 11px;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
}

.subtitle-row__text {
  margin: 0;
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.5;
}

.subtitle-row__text--editable {
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 6px 8px;
  color: var(--text-primary);
  font-family: inherit;
  font-size: 14px;
  resize: none;
  outline: none;
  transition: border-color 0.15s ease;
}

.subtitle-row__text--editable:focus {
  border-color: var(--accent);
}
</style>
