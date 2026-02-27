<script setup lang="ts">
import { useSettings } from '../composables/useSettings'
import IconSun from '../components/icons/IconSun.vue'
import IconMoon from '../components/icons/IconMoon.vue'
import IconMonitor from '../components/icons/IconMonitor.vue'

const { settings, setTheme, setCloseToTray } = useSettings()

const themeOptions = [
  { value: 'dark' as const, label: '深色', icon: IconMoon },
  { value: 'light' as const, label: '浅色', icon: IconSun },
  { value: 'system' as const, label: '跟随系统', icon: IconMonitor },
]
</script>

<template>
  <div class="settings-page">
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
}

.section-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 12px;
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
</style>
