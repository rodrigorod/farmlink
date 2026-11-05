import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Toast } from '@/types'

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<Toast[]>([])

  function addToast(toast: Omit<Toast, 'id'>) {
    const id = Math.random().toString(36).substring(2, 9)
    const newToast: Toast = { ...toast, id, duration: toast.duration ?? 4500 }
    toasts.value.push(newToast)

    if (newToast.duration && newToast.duration > 0) {
      setTimeout(() => {
        removeToast(id)
      }, newToast.duration)
    }
  }

  function removeToast(id: string) {
    toasts.value = toasts.value.filter((t) => t.id !== id)
  }

  function success(title: string, message: string) {
    addToast({ type: 'success', title, message })
  }

  function warning(title: string, message: string) {
    addToast({ type: 'warning', title, message })
  }

  function error(title: string, message: string) {
    addToast({ type: 'error', title, message, duration: 6000 })
  }

  function info(title: string, message: string) {
    addToast({ type: 'info', title, message })
  }

  return {
    toasts,
    addToast,
    removeToast,
    success,
    warning,
    error,
    info,
  }
})
