<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { ReferenceMode } from '@/types/dubbing'
import { invoke } from '@tauri-apps/api/core'

interface NcnVoice { id: string; name: string }

defineProps<{
  modelValue: ReferenceMode
  customAudioPath?: string
  ncnVoiceId?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: ReferenceMode]
  'update:customAudioPath': [value: string]
  'update:ncnVoiceId': [value: string]
}>()

const modes: { value: ReferenceMode; label: string; desc: string }[] = [
  {
    value: 'none',
    label: '内置',
    desc: '使用 N.CN 内置语音，无需外部 TTS 插件',
  },
  {
    value: 'custom',
    label: '自定义',
    desc: '使用您提供的音频文件作为目标声音',
  },
  {
    value: 'clone',
    label: '克隆',
    desc: '从原始视频中提取每段对应的声音作为参考',
  },
]

const ncnVoices = ref<NcnVoice[]>([])
const ncnVoicesLoading = ref(false)

onMounted(async () => {
  ncnVoicesLoading.value = true
  try {
    ncnVoices.value = await invoke<NcnVoice[]>('cmd_list_ncn_voices')
  } catch (e) {
    console.warn('[ReferenceAudioPicker] failed to load NCN voices:', e)
  } finally {
    ncnVoicesLoading.value = false
  }
})

async function pickAudioFile() {
  const selected = await invoke<string | null>('cmd_pick_video_file')
  if (selected) {
    emit('update:customAudioPath', selected)
  }
}
</script>

<template>
  <div class="reference-picker">
    <div
      v-for="mode in modes"
      :key="mode.value"
      class="mode-option"
      :class="{ 'mode-option--active': modelValue === mode.value }"
      @click="emit('update:modelValue', mode.value)"
    >
      <div class="mode-option__radio">
        <div class="mode-option__dot" :class="{ 'mode-option__dot--active': modelValue === mode.value }" />
      </div>
      <div class="mode-option__content">
        <span class="mode-option__label">{{ mode.label }}</span>
        <span class="mode-option__desc">{{ mode.desc }}</span>
      </div>
    </div>

    <!-- 内置模式：声音选择 -->
    <div v-if="modelValue === 'none'" class="extra-row">
      <select
        class="voice-select"
        :value="ncnVoiceId || ''"
        :disabled="ncnVoicesLoading"
        @change="emit('update:ncnVoiceId', ($event.target as HTMLSelectElement).value)"
      >
        <option value="">{{ ncnVoicesLoading ? '加载中...' : '— 请选择声音 —' }}</option>
        <option v-for="v in ncnVoices" :key="v.id" :value="v.id">{{ v.name }}</option>
      </select>
    </div>

    <!-- 自定义模式：音频文件选取 -->
    <div v-if="modelValue === 'custom'" class="extra-row">
      <input
        class="custom-audio-input"
        :value="customAudioPath || ''"
        placeholder="点击选择音频文件..."
        readonly
      />
      <button class="btn-pick" @click="pickAudioFile">选择</button>
    </div>
  </div>
</template>

<style scoped>
.reference-picker {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.mode-option {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 14px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.mode-option:hover {
  background: var(--bg-hover);
}

.mode-option--active {
  border-color: var(--accent);
  background: var(--accent-subtle);
}

.mode-option__radio {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  border: 2px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 1px;
  transition: border-color 0.15s ease;
}

.mode-option--active .mode-option__radio {
  border-color: var(--accent);
}

.mode-option__dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: transparent;
  transition: background 0.15s ease;
}

.mode-option__dot--active {
  background: var(--accent);
}

.mode-option__content {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.mode-option__label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.mode-option__desc {
  font-size: 12px;
  color: var(--text-muted);
}

.extra-row {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

.voice-select,
.custom-audio-input {
  flex: 1;
  padding: 8px 12px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
}

.voice-select:focus,
.custom-audio-input:focus {
  border-color: var(--accent);
}

.voice-select:disabled {
  opacity: 0.5;
}

.btn-pick {
  padding: 8px 16px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s ease;
}

.btn-pick:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
</style>
