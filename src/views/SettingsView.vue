<script setup lang="ts">
import { computed } from 'vue'
import { useSettings } from '../composables/useSettings'
import { useTranscriptionSettings } from '../composables/useTranscriptionSettings'
import type { TranscriptionProviderId } from '../types/transcription'
import IconSun from '../components/icons/IconSun.vue'
import IconMoon from '../components/icons/IconMoon.vue'
import IconMonitor from '../components/icons/IconMonitor.vue'

const { settings, setTheme, setCloseToTray } = useSettings()
const {
  transcriptionSettings,
  activeProvider,
  allProviders,
  setActiveProvider,
  updateActiveConfig,
  validateActive,
  resetConfig,
} = useTranscriptionSettings()

const themeOptions = [
  { value: 'dark' as const, label: '深色', icon: IconMoon },
  { value: 'light' as const, label: '浅色', icon: IconSun },
  { value: 'system' as const, label: '跟随系统', icon: IconMonitor },
]

const configErrors = computed(() => validateActive().errors)

const currentConfig = computed(
  () => transcriptionSettings.value.configs[transcriptionSettings.value.activeProviderId] as Record<string, unknown>
)

function getFieldValue(key: string): unknown {
  return currentConfig.value[key]
}

function onFieldChange(key: string, value: unknown) {
  updateActiveConfig(key, value)
}

function onNumberInput(key: string, raw: string, min: number, max: number) {
  const n = parseInt(raw, 10)
  if (!isNaN(n)) {
    updateActiveConfig(key, Math.min(max, Math.max(min, n)))
  }
}
</script>

<template>
  <div class="settings-page">
    <!-- Appearance -->
    <section class="settings-section">
      <h2 class="section-title">外观</h2>
      <div class="setting-item">
        <span class="setting-label">主题</span>
        <div class="theme-options">
          <button
            v-for="opt in themeOptions"
            :key="opt.value"
            class="theme-card"
            :class="{ active: settings.theme === opt.value }"
            @click="setTheme(opt.value)"
          >
            <component :is="opt.icon" />
            <span>{{ opt.label }}</span>
          </button>
        </div>
      </div>
    </section>

    <!-- System -->
    <section class="settings-section">
      <h2 class="section-title">系统</h2>
      <div class="setting-item row">
        <div class="setting-info">
          <span class="setting-label">关闭时最小化到托盘</span>
          <span class="setting-desc">关闭窗口时应用将最小化到系统托盘，而非退出应用</span>
        </div>
        <label class="toggle">
          <input
            type="checkbox"
            :checked="settings.closeToTray"
            @change="setCloseToTray(($event.target as HTMLInputElement).checked)"
          />
          <span class="toggle-slider" />
        </label>
      </div>
    </section>

    <!-- Transcription Model -->
    <section class="settings-section">
      <h2 class="section-title">转录模型</h2>

      <!-- Provider selection -->
      <div class="setting-item">
        <span class="setting-label">选择服务</span>
        <div class="provider-list">
          <button
            v-for="provider in allProviders"
            :key="provider.id"
            class="provider-card"
            :class="{ active: transcriptionSettings.activeProviderId === provider.id }"
            @click="setActiveProvider(provider.id as TranscriptionProviderId)"
          >
            <span class="provider-card__radio" :class="{ active: transcriptionSettings.activeProviderId === provider.id }" />
            <span class="provider-card__body">
              <span class="provider-card__name">{{ provider.name }}</span>
              <span class="provider-card__desc">{{ provider.description }}</span>
            </span>
            <span
              class="provider-card__badge"
              :class="provider.requiresApiKey ? 'badge--key' : 'badge--free'"
            >
              {{ provider.requiresApiKey ? '需要密钥' : '免费' }}
            </span>
          </button>
        </div>
      </div>

      <!-- Config form -->
      <div class="setting-item config-panel" v-if="activeProvider.configSchema.length > 0">
        <div class="config-panel-header">
          <span class="config-panel-title">{{ activeProvider.name }} 配置</span>
          <button class="reset-btn" @click="resetConfig(activeProvider.id as TranscriptionProviderId)">恢复默认</button>
        </div>

        <div class="config-fields">
          <div
            v-for="field in activeProvider.configSchema"
            :key="field.key"
            class="config-field"
          >
            <label class="field-label">
              {{ field.label }}
              <span v-if="field.required" class="field-required">*</span>
            </label>

            <!-- Toggle -->
            <template v-if="field.type === 'toggle'">
              <label class="toggle">
                <input
                  type="checkbox"
                  :checked="!!getFieldValue(field.key)"
                  @change="onFieldChange(field.key, ($event.target as HTMLInputElement).checked)"
                />
                <span class="toggle-slider" />
              </label>
            </template>

            <!-- Select -->
            <template v-else-if="field.type === 'select'">
              <select
                class="field-select"
                :value="String(getFieldValue(field.key))"
                @change="onFieldChange(field.key, ($event.target as HTMLSelectElement).value)"
              >
                <option
                  v-for="opt in field.options"
                  :key="String(opt.value)"
                  :value="String(opt.value)"
                >{{ opt.label }}</option>
              </select>
            </template>

            <!-- Number -->
            <template v-else-if="field.type === 'number'">
              <input
                type="number"
                class="field-input field-input--number"
                :value="Number(getFieldValue(field.key))"
                :min="field.min"
                :max="field.max"
                @input="onNumberInput(field.key, ($event.target as HTMLInputElement).value, field.min ?? 0, field.max ?? 9999)"
              />
            </template>

            <!-- Text / Password -->
            <template v-else>
              <input
                :type="field.type === 'password' ? 'password' : 'text'"
                class="field-input"
                :value="String(getFieldValue(field.key) ?? '')"
                :placeholder="field.placeholder ?? ''"
                @input="onFieldChange(field.key, ($event.target as HTMLInputElement).value)"
              />
            </template>

            <span v-if="configErrors[field.key]" class="field-error-msg">{{ configErrors[field.key] }}</span>
            <span v-else-if="field.hint" class="field-hint">{{ field.hint }}</span>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings-page {
  display: flex;
  flex-direction: column;
  gap: 24px;
  width: 100%;
}

.settings-section {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 2px;
}

.setting-item {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 16px;
}

.setting-item.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  display: block;
  margin-bottom: 12px;
}

.row .setting-label {
  margin-bottom: 4px;
}

.setting-desc {
  font-size: 12px;
  color: var(--text-secondary);
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

/* Theme cards */
.theme-options {
  display: flex;
  gap: 10px;
}

.theme-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 14px 12px;
  border-radius: 8px;
  border: 2px solid var(--border);
  background: var(--bg-base);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
  font-size: 13px;
  min-width: 0;
}

.theme-card:hover {
  border-color: var(--text-muted);
  color: var(--text-primary);
}

.theme-card.active {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-subtle);
}

/* Toggle switch */
.toggle {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
  flex-shrink: 0;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  inset: 0;
  background: var(--bg-hover);
  border-radius: 12px;
  cursor: pointer;
  transition: background 0.2s ease;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  width: 18px;
  height: 18px;
  left: 3px;
  top: 3px;
  background: var(--text-secondary);
  border-radius: 50%;
  transition: all 0.2s ease;
}

.toggle input:checked + .toggle-slider {
  background: var(--accent);
}

.toggle input:checked + .toggle-slider::before {
  transform: translateX(20px);
  background: #fff;
}

/* Provider list */
.provider-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.provider-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  cursor: pointer;
  transition: all 0.15s ease;
  text-align: left;
  width: 100%;
}

.provider-card:hover {
  border-color: var(--text-muted);
}

.provider-card.active {
  border-color: var(--accent);
  background: var(--accent-subtle);
}

.provider-card__radio {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid var(--border);
  flex-shrink: 0;
  position: relative;
  transition: border-color 0.15s ease;
}

.provider-card__radio.active {
  border-color: var(--accent);
  background: var(--accent);
}

.provider-card__radio.active::after {
  content: '';
  position: absolute;
  width: 6px;
  height: 6px;
  background: #fff;
  border-radius: 50%;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.provider-card__body {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
}

.provider-card__name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.provider-card__desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.4;
}

.provider-card__badge {
  font-size: 11px;
  font-weight: 500;
  padding: 3px 8px;
  border-radius: 4px;
  flex-shrink: 0;
}

.badge--free {
  background: var(--status-success-subtle);
  color: var(--status-success);
}

.badge--key {
  background: var(--status-warning-subtle);
  color: var(--status-warning);
}

/* Config panel */
.config-panel {
  margin-top: 0;
}

.config-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.config-panel-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.reset-btn {
  font-size: 12px;
  color: var(--text-muted);
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.reset-btn:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

/* Config fields */
.config-fields {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.config-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

.field-required {
  color: var(--status-error);
  margin-left: 2px;
}

.field-select,
.field-input {
  padding: 8px 12px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 14px;
  outline: none;
  transition: border-color 0.15s ease;
}

.field-select:focus,
.field-input:focus {
  border-color: var(--accent);
}

.field-input--number {
  width: 120px;
}

.field-hint {
  font-size: 12px;
  color: var(--text-muted);
}

.field-error-msg {
  font-size: 12px;
  color: var(--status-error);
}
</style>
