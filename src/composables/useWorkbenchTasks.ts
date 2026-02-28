import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { WorkbenchTaskListItem, WorkbenchTaskFull, StepStatus } from '@/types/workbench'

const tasks = ref<WorkbenchTaskListItem[]>([])
const isLoading = ref(false)

async function loadTasks(): Promise<void> {
  isLoading.value = true
  try {
    const raw = await invoke<Array<Record<string, unknown>>>('cmd_list_workbench_tasks')
    tasks.value = raw.map((t) => ({
      id: t.id as string,
      name: t.name as string,
      videoName: t.videoName as string,
      videoDuration: t.videoDuration as number,
      currentStep: t.currentStep as number,
      stepStatuses: JSON.parse(t.stepStatuses as string) as StepStatus[],
      sourceLanguage: t.sourceLanguage as string,
      targetLanguage: t.targetLanguage as string,
      status: t.status as 'active' | 'completed',
      createdAt: t.createdAt as string,
      updatedAt: t.updatedAt as string,
    }))
  } finally {
    isLoading.value = false
  }
}

async function getTaskFull(id: string): Promise<WorkbenchTaskFull | null> {
  try {
    return await invoke<WorkbenchTaskFull>('cmd_get_workbench_task_full', { taskId: id })
  } catch {
    return null
  }
}

async function deleteTask(id: string): Promise<void> {
  await invoke('cmd_delete_workbench_task', { taskId: id })
  tasks.value = tasks.value.filter((t) => t.id !== id)
}

export function useWorkbenchTasks() {
  return { tasks, isLoading, loadTasks, getTaskFull, deleteTask }
}
