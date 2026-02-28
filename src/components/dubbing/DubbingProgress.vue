<script setup lang="ts">
import type { DubbingStage, DubbingStatus } from '@/types/dubbing'
import { DUBBING_STAGE_LABELS } from '@/types/dubbing'
import ProgressBar from '@/components/workbench/ProgressBar.vue'

defineProps<{
  stageStatuses: Record<DubbingStage, DubbingStatus>
  stageProgress: Record<DubbingStage, number>
  ttsTotal?: number
  ttsCompleted?: number
  currentMessage?: string
}>()

const STAGE_ORDER: DubbingStage[] = [
  'preprocess', 'media', 'reference', 'tts', 'alignment', 'compose',
]

function stageIcon(status: DubbingStatus): string {
  switch (status) {
    case 'completed': return '✓'
    case 'failed': return '✕'
    case 'running': return '…'
    default: return '○'
  }
}
</script>

<template>
  <div class="dubbing-progress">
    <div v-if="currentMessage" class="current-msg">{{ currentMessage }}</div>

    <div class="stages">
      <div
        v-for="(stage, i) in STAGE_ORDER"
        :key="stage"
        class="stage-row"
        :class="`stage-row--${stageStatuses[stage]}`"
      >
        <div class="stage-header">
          <span class="stage-num">{{ i + 1 }}</span>
          <span class="stage-icon">{{ stageIcon(stageStatuses[stage]) }}</span>
          <span class="stage-label">{{ DUBBING_STAGE_LABELS[stage] }}</span>
          <span v-if="stage === 'tts' && ttsTotal" class="stage-count">
            {{ ttsCompleted }}/{{ ttsTotal }}
          </span>
        </div>
        <ProgressBar
          :percent="stageProgress[stage]"
          :label="DUBBING_STAGE_LABELS[stage]"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.dubbing-progress {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.current-msg {
  font-size: 13px;
  color: var(--text-secondary);
  padding: 8px 12px;
  background: var(--bg-elevated);
  border-radius: 6px;
}

.stages {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.stage-row {
  padding: 10px 14px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  transition: border-color 0.2s;
}

.stage-row--running {
  border-color: var(--accent);
}

.stage-row--completed {
  border-color: var(--status-success);
}

.stage-row--failed {
  border-color: var(--status-error);
}

.stage-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.stage-num {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--bg-hover);
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stage-icon {
  font-size: 14px;
  width: 16px;
  text-align: center;
  color: var(--text-muted);
}

.stage-row--completed .stage-icon {
  color: var(--status-success);
}

.stage-row--failed .stage-icon {
  color: var(--status-error);
}

.stage-row--running .stage-icon {
  color: var(--accent);
}

.stage-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  flex: 1;
}

.stage-count {
  font-size: 12px;
  color: var(--text-muted);
}
</style>
