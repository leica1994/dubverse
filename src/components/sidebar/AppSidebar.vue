<script setup lang="ts">
import { ref, watch } from 'vue'
import SidebarItem from './SidebarItem.vue'
import IconFolder from '../icons/IconFolder.vue'
import IconFilm from '../icons/IconFilm.vue'
import IconLayers from '../icons/IconLayers.vue'
import IconSettings from '../icons/IconSettings.vue'
import IconMenu from '../icons/IconMenu.vue'
import IconChevronRight from '../icons/IconChevronRight.vue'

const STORAGE_KEY = 'dubverse-sidebar-collapsed'

const collapsed = ref(localStorage.getItem(STORAGE_KEY) === 'true')

watch(collapsed, (val) => {
  localStorage.setItem(STORAGE_KEY, String(val))
})

function toggle() {
  collapsed.value = !collapsed.value
}
</script>

<template>
  <aside class="sidebar" :class="{ collapsed }">
    <nav class="sidebar-nav">
      <button class="menu-btn" @click="toggle" :title="collapsed ? '展开侧边栏' : '折叠侧边栏'">
        <IconChevronRight v-if="collapsed" />
        <IconMenu v-else />
      </button>

      <div class="nav-main">
        <SidebarItem to="/" label="工作台" :collapsed="collapsed">
          <template #icon><IconFilm /></template>
        </SidebarItem>
        <SidebarItem to="/batch" label="批量处理" :collapsed="collapsed">
          <template #icon><IconLayers /></template>
        </SidebarItem>
        <SidebarItem to="/projects" label="项目" :collapsed="collapsed">
          <template #icon><IconFolder /></template>
        </SidebarItem>
      </div>

      <div class="nav-bottom">
        <SidebarItem to="/settings" label="设置" :collapsed="collapsed">
          <template #icon><IconSettings /></template>
        </SidebarItem>
      </div>
    </nav>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width-expanded);
  height: 100%;
  background-color: var(--bg-deepest);
  border-right: 1px solid var(--border);
  transition: var(--sidebar-transition);
  flex-shrink: 0;
  overflow: hidden;
}

.sidebar.collapsed {
  width: var(--sidebar-width-collapsed);
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding-top: 0;
}

.nav-main {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
}

.nav-bottom {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-bottom: 8px;
  border-top: 1px solid var(--border);
  margin-top: auto;
  padding-top: 8px;
}

.menu-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 48px;
  margin: 0 8px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 8px;
  transition: color 0.15s ease, background-color 0.15s ease;
  flex-shrink: 0;
}

.menu-btn:hover {
  color: var(--text-secondary);
  background-color: var(--bg-hover);
}
</style>
