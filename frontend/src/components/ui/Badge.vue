<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  variant?: 'synced' | 'outdated' | 'missing' | 'error' | 'neutral' | 'admin'
  className?: string
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'neutral',
  className: '',
})

const badgeStyle = computed(() => {
  switch (props.variant) {
    case 'synced':
      return 'bg-emerald-950/70 text-emerald-400 border border-emerald-800/80'
    case 'outdated':
      return 'bg-amber-950/70 text-amber-400 border border-amber-800/80'
    case 'missing':
      return 'bg-rose-950/70 text-rose-400 border border-rose-800/80'
    case 'error':
      return 'bg-red-950/70 text-red-400 border border-red-800/80'
    case 'admin':
      return 'bg-purple-950/70 text-purple-300 border border-purple-800/80'
    case 'neutral':
    default:
      return 'bg-surface-raised text-muted border border-surface-border'
  }
})
</script>

<template>
  <span
    :class="
      cn(
        'inline-flex items-center gap-1.5 px-2 py-0.5 text-xs font-medium rounded-sm uppercase tracking-wider',
        badgeStyle,
        className
      )
    "
  >
    <slot />
  </span>
</template>
