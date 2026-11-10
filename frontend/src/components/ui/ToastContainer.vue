<script setup lang="ts">
import { useToastStore } from '@/stores/toast'
import {
  CheckCircle2,
  AlertTriangle,
  XCircle,
  Info,
  X,
} from 'lucide-vue-next'

const toastStore = useToastStore()

function getIcon(type: string) {
  switch (type) {
    case 'success':
      return CheckCircle2
    case 'warning':
      return AlertTriangle
    case 'error':
      return XCircle
    default:
      return Info
  }
}

function getCardStyle(_type: string) {
  return 'border border-surface-border bg-surface shadow-swiss-md'
}

function getIconColor(type: string) {
  switch (type) {
    case 'success':
      return 'text-primary'
    case 'warning':
      return 'text-amber-500'
    case 'error':
      return 'text-destructive'
    default:
      return 'text-primary'
  }
}
</script>

<template>
  <div class="fixed bottom-5 right-5 z-50 flex flex-col gap-2 max-w-md w-full pointer-events-none">
    <transition-group
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="transform translate-y-2 opacity-0 scale-95"
      enter-to-class="transform translate-y-0 opacity-100 scale-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="transform opacity-100 scale-100"
      leave-to-class="transform translate-y-1 opacity-0 scale-95"
    >
      <div
        v-for="toast in toastStore.toasts"
        :key="toast.id"
        :class="[
          'pointer-events-auto p-4 border flex items-start gap-3 rounded-lg relative',
          getCardStyle(toast.type),
        ]"
      >
        <component
          :is="getIcon(toast.type)"
          :class="['w-5 h-5 shrink-0 mt-0.5', getIconColor(toast.type)]"
        />
        <div class="flex-1 min-w-0">
          <h4 class="text-xs font-bold uppercase tracking-wider text-foreground font-display">
            {{ toast.title }}
          </h4>
          <p class="text-xs text-muted mt-1 leading-relaxed break-words font-sans">
            {{ toast.message }}
          </p>
        </div>
        <button
          type="button"
          @click="toastStore.removeToast(toast.id)"
          class="text-muted hover:text-foreground p-1 transition-colors rounded-sm"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </transition-group>
  </div>
</template>
