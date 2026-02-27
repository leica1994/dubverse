<script setup lang="ts">
import { ref } from 'vue'
import IconUpload from '@/components/icons/IconUpload.vue'

const emit = defineEmits<{
  drop: [file: File]
}>()

const isDragOver = ref(false)
const fileInput = ref<HTMLInputElement>()

function onDragOver(e: DragEvent) {
  e.preventDefault()
  isDragOver.value = true
}

function onDragLeave() {
  isDragOver.value = false
}

function onDrop(e: DragEvent) {
  e.preventDefault()
  isDragOver.value = false
  const file = e.dataTransfer?.files[0]
  if (file && file.type.startsWith('video/')) {
    emit('drop', file)
  }
}

function onClickSelect() {
  fileInput.value?.click()
}

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (file) {
    emit('drop', file)
    input.value = ''
  }
}
</script>

<template>
  <div
    class="drop-zone"
    :class="{ 'drop-zone--active': isDragOver }"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <IconUpload class="drop-zone__icon" />
    <p class="drop-zone__title">拖拽视频文件到此处</p>
    <p class="drop-zone__hint">或点击下方按钮选择文件</p>
    <button class="drop-zone__btn" @click="onClickSelect">选择文件</button>
    <input
      ref="fileInput"
      type="file"
      accept="video/*"
      class="drop-zone__input"
      @change="onFileChange"
    />
  </div>
</template>

<style scoped>
.drop-zone {
  max-width: 560px;
  margin: 0 auto;
  padding: 48px 32px;
  border: 2px dashed var(--border);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  transition: border-color 0.15s ease, background 0.15s ease;
  cursor: pointer;
}

.drop-zone--active {
  border-color: var(--accent);
  background: var(--accent-subtle);
}

.drop-zone__icon {
  width: 40px;
  height: 40px;
  color: var(--text-muted);
  margin-bottom: 8px;
}

.drop-zone__title {
  font-size: 15px;
  color: var(--text-primary);
  margin: 0;
}

.drop-zone__hint {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0;
}

.drop-zone__btn {
  margin-top: 12px;
  padding: 8px 20px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.drop-zone__btn:hover {
  background: var(--accent-hover);
}

.drop-zone__input {
  display: none;
}
</style>
