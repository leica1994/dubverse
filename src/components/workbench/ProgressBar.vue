<script setup lang="ts">
defineProps<{
  percent: number
  label?: string
  showPercent?: boolean
}>()
</script>

<template>
  <div class="progress-bar">
    <div class="progress-bar__label" v-if="label || showPercent">
      <span v-if="label" class="progress-bar__text">{{ label }}</span>
      <span v-if="showPercent" class="progress-bar__percent">{{ Math.round(percent) }}%</span>
    </div>
    <div class="progress-bar__track">
      <div class="progress-bar__fill" :style="{ width: `${Math.min(100, Math.max(0, percent))}%` }" />
    </div>
  </div>
</template>

<style scoped>
.progress-bar {
  width: 100%;
}

.progress-bar__label {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
  font-size: 13px;
}

.progress-bar__text {
  color: var(--text-secondary);
}

.progress-bar__percent {
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}

.progress-bar__track {
  height: 6px;
  background: var(--bg-hover);
  border-radius: 3px;
  overflow: hidden;
}

.progress-bar__fill {
  height: 100%;
  background: var(--accent);
  border-radius: 3px;
  transition: width 0.3s ease;
}
</style>
