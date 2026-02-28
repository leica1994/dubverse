<script setup lang="ts">
import type { Subtitle } from '@/types/workbench'
import { ref, watch, onMounted, nextTick } from 'vue'

const props = defineProps<{
  subtitle: Subtitle
  editable?: boolean
  loading?: boolean
}>()

const emit = defineEmits<{
  update: [text: string]
}>()

const justUpdated = ref(false)
let flashTimer: ReturnType<typeof setTimeout> | null = null

const textareaRef = ref<HTMLTextAreaElement | null>(null)

function adjustHeight() {
  if (!textareaRef.value) return
  textareaRef.value.style.height = 'auto'
  textareaRef.value.style.height = textareaRef.value.scrollHeight + 'px'
}

onMounted(() => nextTick(adjustHeight))
watch(() => props.subtitle.text, () => nextTick(adjustHeight))

watch(() => props.loading, (newVal, oldVal) => {
  if (oldVal === true && newVal === false) {
    if (flashTimer) clearTimeout(flashTimer)
    justUpdated.value = true
    flashTimer = setTimeout(() => {
      justUpdated.value = false
      flashTimer = null
    }, 800)
  }
})

function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  const ms = Math.floor((seconds % 1) * 100)
  return `${m}:${s.toString().padStart(2, '0')}.${ms.toString().padStart(2, '0')}`
}

function onInput(e: Event) {
  adjustHeight()
  emit('update', (e.target as HTMLTextAreaElement).value)
}
</script>

<template>
  <div class="subtitle-row" :class="{ 'subtitle-row--flash': justUpdated }">
    <span class="subtitle-row__time">
      {{ formatTime(subtitle.startTime) }} - {{ formatTime(subtitle.endTime) }}
    </span>
    <textarea
      v-if="editable"
      ref="textareaRef"
      class="subtitle-row__text subtitle-row__text--editable"
      :value="subtitle.text"
      @input="onInput"
    />
    <p v-else-if="!loading" class="subtitle-row__text">{{ subtitle.text }}</p>
    <p v-else class="subtitle-row__text subtitle-row__text--loading">&#8203;</p>
  </div>
</template>

<style scoped>
.subtitle-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
  transition: background-color 0.3s ease;
}

.subtitle-row--flash {
  background-color: rgba(34, 197, 94, 0.08);
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
  display: block;
  width: 100%;
  background: transparent;
  border: none;
  border-radius: 0;
  padding: 0;
  margin: 0;
  color: var(--text-primary);
  font-family: inherit;
  font-size: 14px;
  line-height: 1.5;
  resize: none;
  outline: none;
  overflow: hidden;
}

.subtitle-row:has(.subtitle-row__text--editable):hover {
  background: var(--bg-hover);
  cursor: text;
}

.subtitle-row:has(.subtitle-row__text--editable:focus) {
  background: color-mix(in srgb, var(--accent) 5%, transparent);
  box-shadow: inset 2px 0 0 var(--accent);
}

.subtitle-row__text--loading {
  background: linear-gradient(90deg, var(--bg-hover) 25%, var(--bg-elevated) 50%, var(--bg-hover) 75%);
  background-size: 200% 100%;
  animation: shimmer 1.4s infinite;
  border-radius: 4px;
  color: transparent;
  height: 20px;
}

@keyframes shimmer {
  0% { background-position: 200% 0 }
  100% { background-position: -200% 0 }
}
</style>
