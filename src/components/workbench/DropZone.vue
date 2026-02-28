<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

const emit = defineEmits<{
  drop: [{ name: string; path: string; file?: File }]
}>()

const isDragOver = ref(false)

const VIDEO_EXTS = /\.(mp4|mov|avi|mkv|webm|flv|wmv|ts|m4v)$/i

const FORMAT_BADGES = ['MP4', 'MOV', 'AVI', 'MKV', 'WebM', 'FLV', 'TS']

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

function onDragLeave(e: DragEvent) {
  // Only reset if leaving the drop zone entirely
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  const x = e.clientX
  const y = e.clientY
  if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
    isDragOver.value = false
  }
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
    @click="onClickSelect"
  >
    <!-- Upload icon -->
    <div class="drop-zone__icon-wrap" :class="{ 'drop-zone__icon-wrap--active': isDragOver }">
      <svg
        class="drop-zone__icon"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polyline points="16 16 12 12 8 16" />
        <line x1="12" y1="12" x2="12" y2="21" />
        <path d="M20.39 18.39A5 5 0 0 0 18 9h-1.26A8 8 0 1 0 3 16.3" />
      </svg>
    </div>

    <!-- Text -->
    <p class="drop-zone__title">
      <span v-if="isDragOver">松开以导入视频</span>
      <span v-else>拖拽视频文件到此处，或<span class="drop-zone__click-hint"> 点击选择</span></span>
    </p>

    <!-- Format badges -->
    <div class="drop-zone__formats">
      <span v-for="fmt in FORMAT_BADGES" :key="fmt" class="drop-zone__badge">{{ fmt }}</span>
    </div>

    <!-- Size limit -->
    <p class="drop-zone__limit">最大支持 4 GB</p>
  </div>
</template>

<style scoped>
.drop-zone {
  max-width: 600px;
  width: 100%;
  margin: 0 auto;
  padding: 52px 40px 44px;
  border: 2px dashed var(--border);
  border-radius: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0;
  background: var(--bg-elevated);
  transition: border-color 0.2s ease, background 0.2s ease, transform 0.2s ease,
              box-shadow 0.2s ease;
  cursor: pointer;
  position: relative;
  overflow: hidden;
}

/* Subtle inner glow on hover */
.drop-zone::before {
  content: '';
  position: absolute;
  inset: 0;
  background: radial-gradient(ellipse at 50% 0%, rgba(99, 102, 241, 0.06) 0%, transparent 65%);
  opacity: 0;
  transition: opacity 0.3s ease;
  pointer-events: none;
}

.drop-zone:hover {
  border-color: var(--text-muted);
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.12);
}

.drop-zone:hover::before {
  opacity: 1;
}

.drop-zone--active {
  border-color: var(--accent) !important;
  border-style: solid;
  background: var(--bg-elevated);
  box-shadow: 0 0 0 4px var(--accent-subtle), 0 4px 24px rgba(99, 102, 241, 0.15);
  transform: scale(1.005);
}

.drop-zone--active::before {
  opacity: 1;
  background: radial-gradient(ellipse at 50% 0%, rgba(99, 102, 241, 0.12) 0%, transparent 70%);
}

/* ── Upload icon ────────────────────────────────────────────────────────── */

.drop-zone__icon-wrap {
  width: 72px;
  height: 72px;
  border-radius: 18px;
  background: var(--bg-hover);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 22px;
  transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1),
              background 0.2s ease,
              border-color 0.2s ease;
  color: var(--text-muted);
}

.drop-zone__icon-wrap--active {
  transform: scale(1.12) translateY(-4px);
  background: var(--accent-subtle);
  border-color: var(--accent);
  color: var(--accent);
}

.drop-zone:hover:not(.drop-zone--active) .drop-zone__icon-wrap {
  transform: translateY(-3px);
  color: var(--text-secondary);
}

.drop-zone__icon {
  width: 36px;
  height: 36px;
}

/* ── Text ───────────────────────────────────────────────────────────────── */

.drop-zone__title {
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
  margin: 0 0 16px;
  text-align: center;
  line-height: 1.5;
}

.drop-zone__click-hint {
  color: var(--accent);
  font-weight: 600;
}

/* ── Format badges ──────────────────────────────────────────────────────── */

.drop-zone__formats {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  justify-content: center;
  margin-bottom: 16px;
}

.drop-zone__badge {
  padding: 3px 9px;
  background: var(--bg-hover);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  letter-spacing: 0.04em;
  transition: border-color 0.2s ease, color 0.2s ease;
}

.drop-zone--active .drop-zone__badge {
  border-color: var(--accent);
  color: var(--accent);
}

/* ── Size limit ─────────────────────────────────────────────────────────── */

.drop-zone__limit {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
  opacity: 0.7;
}
</style>
