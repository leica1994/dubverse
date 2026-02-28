<script setup lang="ts">
import { useWorkbench } from '@/composables/useWorkbench'
import DropZone from './DropZone.vue'
import type { VideoFile } from '@/types/workbench'

const { videoFile, setVideoFile, clearVideoFile } = useWorkbench()

function formatSize(bytes: number): string {
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`
}

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

function onFileDrop({ name, path, file }: { name: string; path: string; file?: File }) {
  const vf: VideoFile = {
    name,
    path,
    size: file?.size ?? 0,
    duration: 0,
    width: 0,
    height: 0,
  }

  if (file) {
    const url = URL.createObjectURL(file)
    const video = document.createElement('video')
    video.preload = 'metadata'
    video.onloadedmetadata = () => {
      vf.duration = video.duration
      vf.width = video.videoWidth
      vf.height = video.videoHeight
      URL.revokeObjectURL(url)
      setVideoFile(vf)
    }
    video.onerror = () => {
      URL.revokeObjectURL(url)
      setVideoFile(vf)
    }
    video.src = url
  } else {
    setVideoFile(vf)
  }
}
</script>

<template>
  <div class="step-upload">
    <!-- Empty state: drop zone -->
    <Transition name="upload-fade" mode="out-in">
      <DropZone v-if="!videoFile" key="dropzone" @drop="onFileDrop" />

      <!-- File selected card -->
      <div v-else key="filecard" class="file-card">
        <!-- Video icon -->
        <div class="file-card__icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <rect x="2" y="2" width="20" height="20" rx="4" ry="4" />
            <polygon points="10 8 16 12 10 16 10 8" fill="currentColor" stroke="none" />
          </svg>
        </div>

        <!-- File info -->
        <div class="file-card__body">
          <p class="file-card__name" :title="videoFile.name">{{ videoFile.name }}</p>

          <div class="file-card__meta">
            <span v-if="videoFile.width" class="file-card__tag">
              {{ videoFile.width }}×{{ videoFile.height }}
            </span>
            <span v-if="videoFile.duration" class="file-card__tag">
              {{ formatDuration(videoFile.duration) }}
            </span>
            <span v-if="videoFile.size > 0" class="file-card__tag">
              {{ formatSize(videoFile.size) }}
            </span>
          </div>
        </div>

        <!-- Divider -->
        <div class="file-card__divider" />

        <!-- Actions -->
        <div class="file-card__actions">
          <button class="btn btn--ghost" @click="clearVideoFile">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="1 4 1 10 7 10" />
              <path d="M3.51 15a9 9 0 1 0 .49-4.51" />
            </svg>
            重新选择
          </button>
          <div class="file-card__ready">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="20 6 9 17 4 12" />
            </svg>
            已就绪
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.step-upload {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: min(320px, 60vh);
  padding: 8px 0;
}

/* ── Upload fade transition ─────────────────────────────────────────────── */

.upload-fade-enter-active,
.upload-fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.upload-fade-enter-from,
.upload-fade-leave-to {
  opacity: 0;
  transform: scale(0.97);
}

/* ── File card ──────────────────────────────────────────────────────────── */

.file-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 36px 32px 28px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 20px;
  max-width: 640px;
  width: 100%;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.1);
  text-align: center;
}

/* ── Video icon ─────────────────────────────────────────────────────────── */

.file-card__icon {
  width: 64px;
  height: 64px;
  border-radius: 16px;
  background: linear-gradient(135deg, var(--accent) 0%, var(--accent-hover) 100%);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 20px;
  flex-shrink: 0;
  box-shadow: 0 4px 14px rgba(99, 102, 241, 0.4);
}

.file-card__icon svg {
  width: 30px;
  height: 30px;
}

/* ── File info ──────────────────────────────────────────────────────────── */

.file-card__body {
  width: 100%;
  margin-bottom: 20px;
}

.file-card__name {
  margin: 0 0 10px;
  font-size: 17px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}

.file-card__meta {
  display: flex;
  gap: 8px;
  justify-content: center;
  flex-wrap: wrap;
}

.file-card__tag {
  padding: 3px 10px;
  background: var(--bg-hover);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
}

/* ── Divider ─────────────────────────────────────────────────────────────── */

.file-card__divider {
  width: 100%;
  height: 1px;
  background: var(--border);
  margin-bottom: 20px;
}

/* ── Actions ─────────────────────────────────────────────────────────────── */

.file-card__actions {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: all 0.15s ease;
}

.btn--ghost {
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border);
}

.btn--ghost:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.file-card__ready {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--status-success);
  background: var(--status-success-subtle);
}
</style>
