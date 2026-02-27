<script setup lang="ts">
import { useWorkbench } from '@/composables/useWorkbench'
import DropZone from './DropZone.vue'
import IconUpload from '@/components/icons/IconUpload.vue'
import type { VideoFile } from '@/types/workbench'

const { videoFile, setVideoFile, clearVideoFile } = useWorkbench()

function formatSize(bytes: number): string {
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`
}

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${m}:${s.toString().padStart(2, '0')}`
}

function onFileDrop(file: File) {
  const vf: VideoFile = {
    name: file.name,
    path: (file as any).path || file.name,
    size: file.size,
    duration: 0,
    width: 0,
    height: 0,
  }
  // Extract video metadata via a temporary video element
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
}
</script>

<template>
  <div class="step-upload">
    <!-- Empty state -->
    <DropZone v-if="!videoFile" @drop="onFileDrop" />

    <!-- File selected -->
    <div v-else class="file-card">
      <div class="file-card__icon">
        <IconUpload />
      </div>
      <div class="file-card__info">
        <p class="file-card__name">{{ videoFile.name }}</p>
        <div class="file-card__meta">
          <span>{{ formatSize(videoFile.size) }}</span>
          <span v-if="videoFile.duration">{{ formatDuration(videoFile.duration) }}</span>
          <span v-if="videoFile.width">{{ videoFile.width }}×{{ videoFile.height }}</span>
        </div>
      </div>
      <button class="btn btn--ghost" @click="clearVideoFile">重新选择</button>
    </div>
  </div>
</template>

<style scoped>
.step-upload {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 320px;
}

.file-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px 24px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 12px;
  max-width: 560px;
  width: 100%;
}

.file-card__icon {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  background: var(--accent-subtle);
  color: var(--accent);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.file-card__info {
  flex: 1;
  min-width: 0;
}

.file-card__name {
  margin: 0;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-card__meta {
  display: flex;
  gap: 12px;
  margin-top: 4px;
  font-size: 12px;
  color: var(--text-muted);
}

.btn {
  padding: 8px 20px;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  border: none;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.btn--ghost {
  background: transparent;
  color: var(--accent);
}

.btn--ghost:hover {
  background: var(--accent-subtle);
}
</style>
