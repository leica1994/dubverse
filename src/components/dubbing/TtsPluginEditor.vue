<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { TtsPlugin } from '@/types/dubbing'
import { TTS_PLUGIN_TYPE_LABELS } from '@/types/dubbing'

const props = defineProps<{
  modelValue: TtsPlugin
  isEdit?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: TtsPlugin]
  save: []
  cancel: []
}>()

const form = ref<TtsPlugin>({ ...props.modelValue })

watch(() => props.modelValue, val => {
  form.value = { ...val }
}, { deep: true })

function onFieldChange() {
  emit('update:modelValue', { ...form.value })
}

// Config JSON parsed/stringified
const ncnConfig = computed(() => {
  try { return JSON.parse(form.value.configJson) } catch { return { voiceId: '' } }
})
const gradioConfig = computed(() => {
  try { return JSON.parse(form.value.configJson) } catch { return { endpoint: '' } }
})
const httpConfig = computed(() => {
  try { return JSON.parse(form.value.configJson) } catch {
    return { url: '', method: 'POST', textKey: 'text', responseType: 'json_base64' }
  }
})

function updateNcnConfig(key: string, val: string) {
  const cfg = { ...ncnConfig.value, [key]: val }
  form.value.configJson = JSON.stringify(cfg)
  onFieldChange()
}

function updateGradioConfig(key: string, val: string) {
  const cfg = { ...gradioConfig.value, [key]: val }
  form.value.configJson = JSON.stringify(cfg)
  onFieldChange()
}

function updateHttpConfig(key: string, val: unknown) {
  const cfg = { ...httpConfig.value, [key]: val }
  form.value.configJson = JSON.stringify(cfg)
  onFieldChange()
}

const PLUGIN_TYPES: TtsPlugin['pluginType'][] = ['ncn', 'gradio', 'http_rest']

function onTypeChange(e: Event) {
  const val = (e.target as HTMLSelectElement).value as TtsPlugin['pluginType']
  form.value.pluginType = val
  // Reset config for new type
  if (val === 'ncn') form.value.configJson = JSON.stringify({ voiceId: '' })
  else if (val === 'gradio') form.value.configJson = JSON.stringify({ endpoint: '' })
  else form.value.configJson = JSON.stringify({ url: '', method: 'POST', textKey: 'text', responseType: 'json_base64' })
  form.value.requiresRef = val === 'gradio'
  onFieldChange()
}
</script>

<template>
  <div class="plugin-editor">
    <div class="field">
      <label class="label">插件名称</label>
      <input v-model="form.name" class="input" placeholder="例：N.CN 标准音" @input="onFieldChange" />
    </div>

    <div class="field">
      <label class="label">类型</label>
      <select class="select" :value="form.pluginType" @change="onTypeChange">
        <option v-for="t in PLUGIN_TYPES" :key="t" :value="t">
          {{ TTS_PLUGIN_TYPE_LABELS[t] }}
        </option>
      </select>
    </div>

    <!-- NCN config -->
    <template v-if="form.pluginType === 'ncn'">
      <div class="field">
        <label class="label">声音 ID (voiceId)</label>
        <input
          :value="ncnConfig.voiceId"
          class="input"
          placeholder="例：xiaoyun"
          @input="updateNcnConfig('voiceId', ($event.target as HTMLInputElement).value)"
        />
        <span class="hint">留空则使用后端默认声音</span>
      </div>
    </template>

    <!-- Gradio config -->
    <template v-else-if="form.pluginType === 'gradio'">
      <div class="field">
        <label class="label">Gradio 接口地址</label>
        <input
          :value="gradioConfig.endpoint"
          class="input"
          placeholder="例：http://127.0.0.1:7860"
          @input="updateGradioConfig('endpoint', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="checkbox-row">
        <input type="checkbox" :checked="form.requiresRef" @change="form.requiresRef = ($event.target as HTMLInputElement).checked; onFieldChange()" />
        <label>需要参考音频</label>
      </div>
    </template>

    <!-- HTTP REST config -->
    <template v-else-if="form.pluginType === 'http_rest'">
      <div class="field">
        <label class="label">请求 URL</label>
        <input
          :value="httpConfig.url"
          class="input"
          placeholder="https://api.example.com/tts"
          @input="updateHttpConfig('url', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="field-row">
        <div class="field">
          <label class="label">Method</label>
          <select class="select" :value="httpConfig.method" @change="updateHttpConfig('method', ($event.target as HTMLSelectElement).value)">
            <option>POST</option><option>GET</option>
          </select>
        </div>
        <div class="field">
          <label class="label">文本字段名</label>
          <input :value="httpConfig.textKey" class="input" placeholder="text" @input="updateHttpConfig('textKey', ($event.target as HTMLInputElement).value)" />
        </div>
      </div>
      <div class="field-row">
        <div class="field">
          <label class="label">响应类型</label>
          <select class="select" :value="httpConfig.responseType" @change="updateHttpConfig('responseType', ($event.target as HTMLSelectElement).value)">
            <option value="json_base64">JSON base64</option>
            <option value="binary">二进制</option>
            <option value="file_url">文件 URL</option>
          </select>
        </div>
        <div class="field">
          <label class="label">响应字段名</label>
          <input :value="httpConfig.responseKey || ''" class="input" placeholder="audio" @input="updateHttpConfig('responseKey', ($event.target as HTMLInputElement).value)" />
        </div>
      </div>
      <div class="checkbox-row">
        <input type="checkbox" :checked="form.requiresRef" @change="form.requiresRef = ($event.target as HTMLInputElement).checked; onFieldChange()" />
        <label>需要参考音频</label>
      </div>
    </template>

    <div class="actions">
      <button class="btn btn--primary" @click="emit('save')">保存</button>
      <button class="btn btn--secondary" @click="emit('cancel')">取消</button>
    </div>
  </div>
</template>

<style scoped>
.plugin-editor {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 5px;
  flex: 1;
}

.field-row {
  display: flex;
  gap: 12px;
}

.label {
  font-size: 13px;
  color: var(--text-secondary);
}

.hint {
  font-size: 12px;
  color: var(--text-muted);
}

.input, .select {
  padding: 8px 12px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 14px;
  outline: none;
}

.input:focus, .select:focus {
  border-color: var(--accent);
}

.checkbox-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--text-secondary);
}

.actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding-top: 4px;
}

.btn {
  padding: 8px 20px;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  border: none;
  transition: all 0.15s ease;
}

.btn--primary { background: var(--accent); color: #fff; }
.btn--primary:hover { background: var(--accent-hover); }
.btn--secondary { background: var(--bg-hover); color: var(--text-secondary); border: 1px solid var(--border); }
.btn--secondary:hover { background: var(--bg-elevated); }
</style>
