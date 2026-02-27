<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const props = defineProps<{
  to: string
  label: string
  collapsed: boolean
}>()

const route = useRoute()
const isActive = computed(() => route.path === props.to)
</script>

<template>
  <router-link :to="to" class="sidebar-item" :class="{ active: isActive, collapsed }">
    <span class="indicator" />
    <span class="icon">
      <slot name="icon" />
    </span>
    <Transition name="fade">
      <span v-if="!collapsed" class="label">{{ label }}</span>
    </Transition>
  </router-link>
</template>

<style scoped>
.sidebar-item {
  position: relative;
  display: flex;
  align-items: center;
  height: var(--nav-item-height);
  padding: 0 16px 0 0;
  color: var(--text-secondary);
  text-decoration: none;
  border-radius: 0 8px 8px 0;
  margin-right: 8px;
  cursor: pointer;
  transition: color 0.15s ease, background-color 0.15s ease;
}

.sidebar-item:hover {
  color: var(--text-primary);
  background-color: var(--bg-hover);
}

.sidebar-item.active {
  color: var(--accent-hover);
  background-color: var(--accent-subtle);
}

.indicator {
  width: var(--nav-indicator-width);
  height: 24px;
  border-radius: 0 2px 2px 0;
  flex-shrink: 0;
  transition: background-color 0.15s ease;
}

.sidebar-item.active .indicator {
  background-color: var(--accent);
}

.icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  flex-shrink: 0;
}

.sidebar-item.collapsed .icon {
  width: 61px;
}

.label {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
