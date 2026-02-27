<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import IconUpload from '@/components/icons/IconUpload.vue'

const emit = defineEmits<{
  drop: [{ name: string; path: string; file?: File }]
}>()

const isDragOver = ref(false)

const VIDEO_EXTS = /\.(mp4|mov|avi|mkv|webm|flv|wmv|ts|m4v)$/i

let unlistenDrop: (() => void) | undefined

onMounted(async () => {
  unlistenDrop = await getCurrentWindow().onDragDropEvent((event) => {
    if (event.payload.type === 'enter' || event.payload.type === 'over') {
      isDragOver.value = true
    } else if (event.payload.type === 'leave') {
      isDragOver.value = false
    } else if (event.payload.type === 'drop') {
      isDragOver.value = false
      const paths: string[] = (event.payload as any).paths ?? []
      const videoPath = paths.find(p => VIDEO_EXTS.test(p))
      if (videoPath) {
        const name = videoPath.replace(/\\/g, '/').split('/').pop() ?? videoPath
        emit('drop', { name, path: videoPath })
      }
    }
  })
})

onUnmounted(() => unlistenDrop?.())

// Keep browser drag handlers for visual feedback only
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
  // Path is handled by the Tauri native onDragDropEvent listener above
}

async function onClickSelect() {
  const path = await invoke<string | null>('cmd_pick_video_file')
  if (path) {
    const name = path.replace(/\\/g, '/').split('/').pop() ?? path
    emit('drop', { name, path })
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
</style>
