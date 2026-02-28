<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useTtsPlugins } from '@/composables/useTtsPlugins'
import { TTS_PLUGIN_TYPE_LABELS } from '@/types/dubbing'

const props = defineProps<{
  modelValue: string | undefined
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string | undefined]
}>()

const { ttsPlugins, loadTtsPlugins, testTtsPlugin } = useTtsPlugins()
const testing = ref(false)
const testError = ref('')

onMounted(loadTtsPlugins)

const enabledPlugins = computed(() =>
  ttsPlugins.value.filter(p => p.isEnabled)
)

async function onTest() {
  if (!props.modelValue) return
  testing.value = true
  testError.value = ''
  try {
    const b64 = await testTtsPlugin(props.modelValue)
    const audio = new Audio(`data:audio/mp3;base64,${b64}`)
    audio.play()
  } catch (err) {
    testError.value = String(err)
  } finally {
    testing.value = false
  }
}
</script>

<template>
  <div class="plugin-selector">
    <div class="selector-row">
      <select
        class="plugin-select"
        :value="modelValue ?? ''"
        @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value || undefined)"
      >
        <option value="">— 请选择 TTS 提供商 —</option>
        <option
          v-for="plugin in enabledPlugins"
          :key="plugin.id"
          :value="plugin.id"
        >
          {{ plugin.name }} ({{ TTS_PLUGIN_TYPE_LABELS[plugin.pluginType] }})
        </option>
      </select>

      <button
        class="btn-test"
        :disabled="!modelValue || testing"
        @click="onTest"
      >
        {{ testing ? '测试中...' : '测试 ▶' }}
      </button>
    </div>

    <div v-if="testError" class="test-error">{{ testError }}</div>

    <div v-if="!enabledPlugins.length" class="empty-hint">
      尚无可用 TTS 插件，请前往<strong>设置 → TTS 插件</strong>添加。
    </div>
  </div>
</template>

<style scoped>
.plugin-selector {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.selector-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.plugin-select {
  flex: 1;
  padding: 8px 12px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 14px;
  outline: none;
  transition: border-color 0.15s ease;
}

.plugin-select:focus {
  border-color: var(--accent);
}

.btn-test {
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

.btn-test:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn-test:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.test-error {
  font-size: 12px;
  color: var(--status-error);
  padding: 6px 10px;
  background: var(--status-error-subtle);
  border-radius: 6px;
}

.empty-hint {
  font-size: 13px;
  color: var(--text-muted);
  padding: 10px 14px;
  background: var(--bg-elevated);
  border: 1px dashed var(--border);
  border-radius: 8px;
}
</style>
