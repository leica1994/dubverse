<script setup lang="ts">
import { ref } from 'vue'
import type { Subtitle } from '@/types/workbench'
import SubtitleRow from './SubtitleRow.vue'

defineProps<{
  original: Subtitle[]
  translated: Subtitle[]
}>()

const emit = defineEmits<{
  updateTranslation: [id: number, text: string]
}>()

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
  if (leftCol.value && rightCol.value) {
    syncScroll(leftCol.value, rightCol.value)
  }
}

function onRightScroll() {
  if (rightCol.value && leftCol.value) {
    syncScroll(rightCol.value, leftCol.value)
  }
}

function onUpdateTranslation(id: number, text: string) {
  emit('updateTranslation', id, text)
}
</script>

<template>
  <div class="subtitle-editor">
    <div class="subtitle-editor__header">
      <span class="subtitle-editor__title">原文</span>
      <span class="subtitle-editor__title">译文（可编辑）</span>
    </div>
    <div class="subtitle-editor__body">
      <div ref="leftCol" class="subtitle-editor__col" @scroll="onLeftScroll">
        <SubtitleRow
          v-for="sub in original"
          :key="sub.id"
          :subtitle="sub"
        />
      </div>
      <div ref="rightCol" class="subtitle-editor__col" @scroll="onRightScroll">
        <SubtitleRow
          v-for="sub in translated"
          :key="sub.id"
          :subtitle="sub"
          editable
          @update="(text) => onUpdateTranslation(sub.id, text)"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.subtitle-editor {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: hidden;
  height: 100%;
}

.subtitle-editor__header {
  display: grid;
  grid-template-columns: 1fr 1fr;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elevated);
}

.subtitle-editor__title {
  padding: 10px 16px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
}

.subtitle-editor__title:first-child {
  border-right: 1px solid var(--border);
}

.subtitle-editor__body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  flex: 1;
  min-height: 0;
}

.subtitle-editor__col {
  overflow-y: auto;
  max-height: 400px;
}

.subtitle-editor__col:first-child {
  border-right: 1px solid var(--border);
}
</style>
