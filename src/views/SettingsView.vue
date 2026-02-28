<script setup lang="ts">
import { computed, ref } from 'vue'
import { useSettings } from '../composables/useSettings'
import { useTranscriptionSettings } from '../composables/useTranscriptionSettings'
import { useTranslationSettings } from '../composables/useTranslationSettings'
import { useAiConfigs } from '../composables/useAiConfigs'
import type { TranscriptionProviderId } from '../types/transcription'
import type { AiConfig } from '../types/ai-config'
import { AI_CONFIG_DEFAULTS } from '../types/ai-config'
import { PROMPT_DEFAULTS } from '../types/translation-prompts'
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

const { translationSettings } = useTranslationSettings()

const themeOptions = [
  { value: 'dark' as const, label: '深色', icon: IconMoon },
  { value: 'light' as const, label: '浅色', icon: IconSun },
  { value: 'system' as const, label: '跟随系统', icon: IconMonitor },
]

const configErrors = computed(() => validateActive().errors)

// ── AI Configs ────────────────────────────────────────────────────────────────

const {
  aiConfigs,
  createAiConfig,
  updateAiConfig,
  deleteAiConfig,
  setDefaultAiConfig,
  testAiConnection,
} = useAiConfigs()

type FormMode = 'create' | 'edit'
const aiFormVisible = ref(false)
const aiFormMode = ref<FormMode>('create')
const aiFormData = ref<AiConfig>({ id: '', ...AI_CONFIG_DEFAULTS })
const aiFormShowKey = ref(false)
const aiFormTesting = ref(false)
const aiFormTestResult = ref('')
const aiFormTestError = ref('')
const aiFormSaving = ref(false)

function openCreateForm() {
  aiFormMode.value = 'create'
  aiFormData.value = { id: crypto.randomUUID(), ...AI_CONFIG_DEFAULTS }
  aiFormShowKey.value = false
  aiFormTestResult.value = ''
  aiFormTestError.value = ''
  aiFormVisible.value = true
}

function openEditForm(config: AiConfig) {
  aiFormMode.value = 'edit'
  aiFormData.value = { ...config }
  aiFormShowKey.value = false
  aiFormTestResult.value = ''
  aiFormTestError.value = ''
  aiFormVisible.value = true
}

function cancelForm() {
  aiFormVisible.value = false
}

async function saveAiForm() {
  aiFormSaving.value = true
  try {
    if (aiFormMode.value === 'create') {
      await createAiConfig(aiFormData.value)
    } else {
      await updateAiConfig(aiFormData.value)
    }
    aiFormVisible.value = false
  } catch (err) {
    console.error('[SettingsView] save ai config failed', err)
  } finally {
    aiFormSaving.value = false
  }
}

async function onTestConnection() {
  aiFormTesting.value = true
  aiFormTestResult.value = ''
  aiFormTestError.value = ''
  try {
    const msg = await testAiConnection(
      aiFormData.value.baseUrl,
      aiFormData.value.apiKey,
      aiFormData.value.model,
    )
    aiFormTestResult.value = msg
  } catch (err) {
    aiFormTestError.value = String(err)
  } finally {
    aiFormTesting.value = false
  }
}

async function onDeleteConfig(id: string) {
  await deleteAiConfig(id)
}

async function onSetDefault(id: string) {
  await setDefaultAiConfig(id)
}

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

const promptSectionCollapsed = ref(true)

const showPassword = ref<Record<string, boolean>>({})
function togglePassword(key: string) {
  showPassword.value[key] = !showPassword.value[key]
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
    <section class="settings-section">      <h2 class="section-title">转录模型</h2>

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
              <div v-if="field.type === 'password'" class="password-wrapper">
                <input
                  :type="showPassword[field.key] ? 'text' : 'password'"
                  class="field-input"
                  :value="String(getFieldValue(field.key) ?? '')"
                  :placeholder="field.placeholder ?? ''"
                  @input="onFieldChange(field.key, ($event.target as HTMLInputElement).value)"
                />
                <button type="button" class="password-toggle" @click="togglePassword(field.key)"
                  :title="showPassword[field.key] ? '隐藏' : '显示'">
                  <svg v-if="showPassword[field.key]" xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/>
                  </svg>
                  <svg v-else xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/>
                  </svg>
                </button>
              </div>
              <input v-else type="text" class="field-input"
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

    <!-- AI Models -->
    <section class="settings-section">
      <h2 class="section-title">AI 模型</h2>

      <!-- Config list -->
      <div class="setting-item" v-if="aiConfigs.length > 0">
        <div class="ai-config-list">
          <div
            v-for="cfg in aiConfigs"
            :key="cfg.id"
            class="ai-config-item"
          >
            <div class="ai-config-info">
              <span class="ai-config-title">{{ cfg.title || '未命名' }}</span>
              <span class="ai-config-meta">{{ cfg.model }} · {{ cfg.baseUrl.replace(/https?:\/\//, '') }}</span>
            </div>
            <div class="ai-config-actions">
              <span v-if="cfg.isDefault" class="badge--default">默认</span>
              <button
                v-else
                class="ai-action-btn"
                @click="onSetDefault(cfg.id)"
                title="设为默认"
              >设为默认</button>
              <button class="ai-action-btn" @click="openEditForm(cfg)" title="编辑">编辑</button>
              <button
                class="ai-action-btn ai-action-btn--danger"
                @click="onDeleteConfig(cfg.id)"
                title="删除"
              >删除</button>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div class="setting-item ai-empty" v-else>
        <span class="ai-empty-text">暂无 AI 配置，点击下方按钮添加</span>
      </div>

      <!-- Add button -->
      <button class="add-ai-btn" @click="openCreateForm" v-if="!aiFormVisible">
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        添加 AI 配置
      </button>

      <!-- Inline form -->
      <div class="setting-item ai-form" v-if="aiFormVisible">
        <div class="config-panel-header">
          <span class="config-panel-title">{{ aiFormMode === 'create' ? '添加 AI 配置' : '编辑 AI 配置' }}</span>
          <button class="reset-btn" @click="cancelForm">取消</button>
        </div>

        <div class="config-fields">
          <!-- Title -->
          <div class="config-field">
            <label class="field-label">名称 <span class="field-required">*</span></label>
            <input type="text" class="field-input" v-model="aiFormData.title" placeholder="如：OpenAI GPT-4o" />
          </div>

          <!-- Base URL -->
          <div class="config-field">
            <label class="field-label">API 地址</label>
            <input type="text" class="field-input" v-model="aiFormData.baseUrl" placeholder="https://api.openai.com/v1" />
          </div>

          <!-- API Key -->
          <div class="config-field">
            <label class="field-label">API Key</label>
            <div class="password-wrapper">
              <input
                :type="aiFormShowKey ? 'text' : 'password'"
                class="field-input"
                v-model="aiFormData.apiKey"
                placeholder="sk-..."
              />
              <button type="button" class="password-toggle" @click="aiFormShowKey = !aiFormShowKey" :title="aiFormShowKey ? '隐藏' : '显示'">
                <svg v-if="aiFormShowKey" xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/>
                </svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/>
                </svg>
              </button>
            </div>
          </div>

          <!-- Model -->
          <div class="config-field">
            <label class="field-label">模型</label>
            <input type="text" class="field-input" v-model="aiFormData.model" placeholder="gpt-4o-mini" />
          </div>

          <!-- Numeric row -->
          <div class="ai-form-row">
            <div class="config-field">
              <label class="field-label">并发数 (1–10)</label>
              <input
                type="number" class="field-input field-input--number"
                v-model.number="aiFormData.concurrentLimit"
                min="1" max="10"
              />
            </div>
            <div class="config-field">
              <label class="field-label">超时 (秒, 0–600)</label>
              <input
                type="number" class="field-input field-input--number"
                v-model.number="aiFormData.requestTimeout"
                min="0" max="600"
              />
            </div>
            <div class="config-field">
              <label class="field-label">限速 (次/分, 0=不限)</label>
              <input
                type="number" class="field-input field-input--number"
                v-model.number="aiFormData.rateLimit"
                min="0" max="1000"
              />
            </div>
          </div>

          <!-- Test + Save row -->
          <div class="ai-form-actions">
            <button class="ai-test-btn" @click="onTestConnection" :disabled="aiFormTesting">
              {{ aiFormTesting ? '测试中…' : '测试连接' }}
            </button>
            <span v-if="aiFormTestResult" class="ai-test-ok">{{ aiFormTestResult }}</span>
            <span v-if="aiFormTestError" class="ai-test-err">{{ aiFormTestError }}</span>
            <button
              class="ai-save-btn"
              @click="saveAiForm"
              :disabled="aiFormSaving || !aiFormData.title"
            >
              {{ aiFormSaving ? '保存中…' : '保存' }}
            </button>
          </div>
        </div>
      </div>
    </section>

    <!-- Translation Settings -->
    <section class="settings-section">
      <h2 class="section-title">翻译设置</h2>

      <div class="setting-item">
        <div class="config-fields">
          <!-- Correction toggle -->
          <div class="config-field">
            <div class="field-row">
              <div class="field-row__info">
                <label class="field-label">校正阶段</label>
                <span class="field-hint">翻译前先校正 ASR 转录错误</span>
              </div>
              <label class="toggle">
                <input type="checkbox" v-model="translationSettings.correction" />
                <span class="toggle-slider" />
              </label>
            </div>
          </div>

          <!-- Optimization toggle -->
          <div class="config-field">
            <div class="field-row">
              <div class="field-row__info">
                <label class="field-label">优化阶段</label>
                <span class="field-hint">翻译后润色提升流畅度</span>
              </div>
              <label class="toggle">
                <input type="checkbox" v-model="translationSettings.optimization" />
                <span class="toggle-slider" />
              </label>
            </div>
          </div>

          <!-- Prompt type -->
          <div class="config-field">
            <label class="field-label">翻译模式</label>
            <select class="field-select" v-model="translationSettings.promptType">
              <option value="standard">标准翻译</option>
              <option value="reflective">反思翻译（更高质量，更慢）</option>
            </select>
          </div>

          <!-- Batch size -->
          <div class="config-field">
            <label class="field-label">每批字幕数</label>
            <input
              type="number" class="field-input field-input--number"
              v-model.number="translationSettings.batchSize"
              min="1" max="100"
            />
            <span class="field-hint">每次 API 调用处理的字幕条数，建议 10-50</span>
          </div>

          <!-- World building -->
          <div class="config-field">
            <label class="field-label">世界观 / 背景设定</label>
            <textarea
              class="field-textarea" rows="3"
              v-model="translationSettings.worldBuilding"
              placeholder="描述视频的背景、领域、角色等信息..."
            />
          </div>

          <!-- Writing style -->
          <div class="config-field">
            <label class="field-label">文风要求</label>
            <textarea
              class="field-textarea" rows="2"
              v-model="translationSettings.writingStyle"
              placeholder="如：口语化、正式、学术..."
            />
          </div>

          <!-- Glossary -->
          <div class="config-field">
            <label class="field-label">术语表</label>
            <textarea
              class="field-textarea" rows="3"
              v-model="translationSettings.glossary"
              placeholder="source → target，每行一条"
            />
          </div>

          <!-- Forbidden -->
          <div class="config-field">
            <label class="field-label">禁用词</label>
            <textarea
              class="field-textarea" rows="2"
              v-model="translationSettings.forbidden"
              placeholder="不希望出现的词汇或翻译，每行一条"
            />
          </div>

          <!-- Examples -->
          <div class="config-field">
            <label class="field-label">翻译示例</label>
            <textarea
              class="field-textarea" rows="3"
              v-model="translationSettings.examples"
              placeholder="提供原文 → 译文示例，帮助 AI 理解期望的翻译风格"
            />
          </div>

          <!-- Custom prompt -->
          <div class="config-field">
            <label class="field-label">自定义提示词</label>
            <textarea
              class="field-textarea" rows="3"
              v-model="translationSettings.customPrompt"
              placeholder="额外的翻译指令..."
            />
          </div>

          <!-- Advanced prompt config (collapsible) -->
          <div class="prompt-section">
            <button class="prompt-section__toggle" @click="promptSectionCollapsed = !promptSectionCollapsed">
              <svg
                class="prompt-section__arrow"
                :class="{ expanded: !promptSectionCollapsed }"
                xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24"
                fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              ><polyline points="9 18 15 12 9 6"/></svg>
              <span>高级提示词配置</span>
            </button>

            <div v-if="!promptSectionCollapsed" class="prompt-section__body">
              <!-- Correction prompt -->
              <div class="config-field">
                <div class="prompt-field-header">
                  <label class="field-label">校正阶段提示词</label>
                  <button
                    v-if="translationSettings.promptCorrection"
                    class="reset-btn" @click="translationSettings.promptCorrection = ''"
                  >恢复默认</button>
                </div>
                <textarea
                  class="field-textarea" rows="4"
                  v-model="translationSettings.promptCorrection"
                  :placeholder="PROMPT_DEFAULTS.correction"
                />
              </div>

              <!-- Standard prompt -->
              <div class="config-field">
                <div class="prompt-field-header">
                  <label class="field-label">标准翻译提示词</label>
                  <button
                    v-if="translationSettings.promptStandard"
                    class="reset-btn" @click="translationSettings.promptStandard = ''"
                  >恢复默认</button>
                </div>
                <textarea
                  class="field-textarea" rows="4"
                  v-model="translationSettings.promptStandard"
                  :placeholder="PROMPT_DEFAULTS.standard"
                />
              </div>

              <!-- Reflective prompt -->
              <div class="config-field">
                <div class="prompt-field-header">
                  <label class="field-label">反思翻译提示词</label>
                  <button
                    v-if="translationSettings.promptReflective"
                    class="reset-btn" @click="translationSettings.promptReflective = ''"
                  >恢复默认</button>
                </div>
                <textarea
                  class="field-textarea" rows="5"
                  v-model="translationSettings.promptReflective"
                  :placeholder="PROMPT_DEFAULTS.reflective"
                />
              </div>

              <!-- Optimize prompt -->
              <div class="config-field">
                <div class="prompt-field-header">
                  <label class="field-label">优化阶段提示词</label>
                  <button
                    v-if="translationSettings.promptOptimize"
                    class="reset-btn" @click="translationSettings.promptOptimize = ''"
                  >恢复默认</button>
                </div>
                <textarea
                  class="field-textarea" rows="4"
                  v-model="translationSettings.promptOptimize"
                  :placeholder="PROMPT_DEFAULTS.optimize"
                />
              </div>
            </div>
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

.password-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.password-wrapper .field-input {
  flex: 1;
  padding-right: 38px;
}

.password-wrapper input::-ms-reveal,
.password-wrapper input::-ms-clear {
  display: none;
}

.password-toggle {
  position: absolute;
  right: 10px;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-muted);
  padding: 0;
  display: flex;
  align-items: center;
  transition: color 0.15s ease;
}

.password-toggle:hover {
  color: var(--text-secondary);
}

/* AI Configs */
.ai-config-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.ai-config-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-base);
}

.ai-config-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
}

.ai-config-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.ai-config-meta {
  font-size: 12px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ai-config-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.badge--default {
  font-size: 11px;
  font-weight: 500;
  padding: 3px 8px;
  border-radius: 4px;
  background: var(--accent-subtle);
  color: var(--accent);
}

.ai-action-btn {
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
}

.ai-action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.ai-action-btn--danger:hover {
  border-color: var(--status-error);
  color: var(--status-error);
}

.ai-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.ai-empty-text {
  font-size: 13px;
  color: var(--text-muted);
}

.add-ai-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 8px;
  border: 1px dashed var(--border);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 13px;
  transition: all 0.15s ease;
  align-self: flex-start;
}

.add-ai-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.ai-form {
  margin-top: 0;
}

.ai-form-row {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.ai-form-row .config-field {
  flex: 1;
  min-width: 120px;
}

.ai-form-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  margin-top: 4px;
}

.ai-test-btn {
  padding: 7px 14px;
  border-radius: 7px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 13px;
  transition: all 0.15s ease;
}

.ai-test-btn:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--accent);
}

.ai-test-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.ai-test-ok {
  font-size: 12px;
  color: var(--status-success);
}

.ai-test-err {
  font-size: 12px;
  color: var(--status-error);
  flex: 1;
}

.ai-save-btn {
  margin-left: auto;
  padding: 7px 20px;
  border-radius: 7px;
  border: none;
  background: var(--accent);
  color: #fff;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: opacity 0.15s ease;
}

.ai-save-btn:hover:not(:disabled) {
  opacity: 0.85;
}

.ai-save-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.field-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.field-row__info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.field-row__info .field-label {
  margin-bottom: 0;
}

.field-textarea {
  padding: 8px 12px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 14px;
  font-family: inherit;
  outline: none;
  resize: vertical;
  transition: border-color 0.15s ease;
}

.field-textarea:focus {
  border-color: var(--accent);
}

/* Prompt section (collapsible) */
.prompt-section {
  border-top: 1px solid var(--border);
  padding-top: 12px;
}

.prompt-section__toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  padding: 4px 0;
  transition: color 0.15s ease;
}

.prompt-section__toggle:hover {
  color: var(--text-primary);
}

.prompt-section__arrow {
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.prompt-section__arrow.expanded {
  transform: rotate(90deg);
}

.prompt-section__body {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-top: 12px;
}

.prompt-field-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
</style>
