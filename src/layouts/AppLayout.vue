<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import AppSidebar from '../components/sidebar/AppSidebar.vue'
import { useSettings } from '../composables/useSettings'
import IconSun from '../components/icons/IconSun.vue'
import IconMoon from '../components/icons/IconMoon.vue'
import IconMinimize from '../components/icons/IconMinimize.vue'
import IconMaximize from '../components/icons/IconMaximize.vue'
import IconClose from '../components/icons/IconClose.vue'
import appIcon from '../assets/app-icon.png'

const route = useRoute()
const { settings, setTheme } = useSettings()
const appWindow = getCurrentWindow()

const pageTitle = computed(() => (route.meta?.title as string) || '')

function toggleTheme() {
  const next = settings.value.theme === 'dark' ? 'light'
    : settings.value.theme === 'light' ? 'dark'
    : 'dark'
  setTheme(next)
}

const themeIcon = computed(() => {
  if (settings.value.theme === 'system') {
    return document.documentElement.getAttribute('data-theme') === 'dark' ? IconMoon : IconSun
  }
  return settings.value.theme === 'dark' ? IconMoon : IconSun
})

async function minimize() { await appWindow.minimize() }
async function toggleMaximize() { await appWindow.toggleMaximize() }
async function close() { await appWindow.close() }
</script>

<template>
  <div class="app-shell">
    <header class="titlebar" data-tauri-drag-region>
      <div class="titlebar-brand" data-tauri-drag-region>
        <img :src="appIcon" class="titlebar-icon" alt="" />
        <span class="titlebar-title" data-tauri-drag-region>Dubverse</span>
      </div>
      <div class="titlebar-actions">
        <button class="titlebar-btn" @click="toggleTheme" title="切换主题">
          <component :is="themeIcon" />
        </button>
        <button class="titlebar-btn" @click="minimize" title="最小化">
          <IconMinimize />
        </button>
        <button class="titlebar-btn" @click="toggleMaximize" title="最大化">
          <IconMaximize />
        </button>
        <button class="titlebar-btn close-btn" @click="close" title="关闭">
          <IconClose />
        </button>
      </div>
    </header>
    <div class="app-layout">
      <AppSidebar />
      <div class="main-wrapper">
        <div v-if="!route.meta?.flush" class="page-header">
          <h1 class="page-title">{{ pageTitle }}</h1>
        </div>
        <main class="main-content" :class="{ 'main-content--flush': route.meta?.flush }">
          <router-view />
        </main>
      </div>
    </div>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100vh;
  overflow: hidden;
}

.titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 36px;
  padding-left: 12px;
  background-color: var(--bg-deepest);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  user-select: none;
}

.titlebar-brand {
  display: flex;
  align-items: center;
}

.titlebar-icon {
  width: 18px;
  height: 18px;
  margin-right: 6px;
}

.titlebar-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
}

.titlebar-actions {
  display: flex;
  align-items: center;
  height: 100%;
}

.titlebar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background-color 0.1s ease, color 0.1s ease;
}

.titlebar-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.close-btn:hover {
  background-color: #e81123;
  color: #fff;
}

.app-layout {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.main-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background-color: var(--bg-base);
}

.page-header {
  display: flex;
  align-items: center;
  height: 48px;
  padding: 0 24px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.page-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

.main-content--flush {
  overflow: hidden;
  padding: 0;
  container-type: inline-size;
  container-name: workbench-root;
}
</style>
