<script setup lang="ts">
import type { Subtitle } from '@/types/workbench'
import SubtitleRow from './SubtitleRow.vue'

defineProps<{
  original: Subtitle[]
  translated: Subtitle[]
}>()

const emit = defineEmits<{
  updateTranslation: [id: number, text: string]
}>()

function onUpdateTranslation(id: number, text: string) {
  emit('updateTranslation', id, text)
}
</script>

<template>
  <div class="subtitle-editor">
    <div class="subtitle-editor__body">
      <div class="subtitle-editor__header">
        <span class="subtitle-editor__title">原文</span>
        <span class="subtitle-editor__title">译文（可编辑）</span>
      </div>
      <div v-for="(orig, i) in original" :key="orig.id" class="subtitle-pair">
        <div class="subtitle-pair__cell">
          <SubtitleRow :subtitle="orig" />
        </div>
        <div class="subtitle-pair__cell subtitle-pair__cell--right">
          <SubtitleRow
            :subtitle="translated[i]"
            editable
            @update="(text) => onUpdateTranslation(orig.id, text)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.subtitle-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* ── Header ─────────────────────────────────────────────────────────────── */

.subtitle-editor__header {
  display: grid;
  grid-template-columns: 1fr 1fr;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elevated);
  position: sticky;
  top: 0;
  z-index: 1;
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

/* ── Body (single scroll container) ─────────────────────────────────────── */

.subtitle-editor__body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

/* ── Paired row ─────────────────────────────────────────────────────────── */

.subtitle-pair {
  display: grid;
  grid-template-columns: 1fr 1fr;
  border-bottom: 1px solid var(--border);
}

.subtitle-pair__cell--right {
  border-left: 1px solid var(--border);
}

/* SubtitleRow内部不再需要自己的 border-bottom */
.subtitle-pair :deep(.subtitle-row) {
  border-bottom: none;
  height: 100%;
}

/* ── Narrow width: stack vertically ──────────────────────────────────────── */

@container workbench-root (max-width: 560px) {
  .subtitle-editor__header,
  .subtitle-pair {
    grid-template-columns: 1fr;
  }
  .subtitle-editor__title:first-child {
    border-right: none;
    border-bottom: 1px solid var(--border);
  }
  .subtitle-pair__cell--right {
    border-left: none;
    border-top: 1px solid var(--border);
  }
}
</style>
