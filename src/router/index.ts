import { createRouter, createWebHashHistory } from 'vue-router'
import AppLayout from '../layouts/AppLayout.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      component: AppLayout,
      children: [
        {
          path: '',
          name: 'workspace',
          meta: { title: '工作台' },
          component: () => import('../views/WorkspaceView.vue'),
        },
        {
          path: 'batch',
          name: 'batch',
          meta: { title: '批量处理' },
          component: () => import('../views/BatchView.vue'),
        },
        {
          path: 'projects',
          name: 'projects',
          meta: { title: '项目' },
          component: () => import('../views/ProjectsView.vue'),
        },
        {
          path: 'settings',
          name: 'settings',
          meta: { title: '设置' },
          component: () => import('../views/SettingsView.vue'),
        },
      ],
    },
  ],
})

export default router
