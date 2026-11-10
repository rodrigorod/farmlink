<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  variant?: 'synced' | 'outdated' | 'missing' | 'error' | 'neutral' | 'admin' | 'brand'
  size?: 'sm' | 'md'
  pill?: boolean
  className?: string
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'neutral',
  size: 'md',
  pill: true,
  className: '',
})

const badgeStyle = computed(() => {
  switch (props.variant) {
    case 'synced':
      return 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-400 border border-emerald-500/30'
    case 'outdated':
      return 'bg-amber-500/10 text-amber-800 dark:text-amber-400 border border-amber-500/30'
    case 'missing':
      return 'bg-rose-500/10 text-rose-700 dark:text-rose-400 border border-rose-500/30'
    case 'error':
      return 'bg-red-500/10 text-red-700 dark:text-red-400 border border-red-500/30'
    case 'admin':
      return 'bg-brand-dark text-white border border-brand-dark dark:bg-purple-950/60 dark:text-purple-300 dark:border-purple-800'
    case 'brand':
      return 'bg-primary text-white border border-primary'
    case 'neutral':
    default:
      return 'bg-surface-raised text-foreground/80 border border-surface-border'
  }
})

const sizeStyle = computed(() => {
  return props.size === 'sm'
    ? 'px-2 py-0.5 text-[10px]'
    : 'px-2.5 py-0.5 text-[11px]'
})
</script>

<template>
  <span
    :class="
      cn(
        'inline-flex items-center gap-1.5 font-bold uppercase tracking-wider select-none',
        pill ? 'rounded-full' : 'rounded-sm',
        badgeStyle,
        sizeStyle,
        className
      )
    "
  >
    <slot />
  </span>
</template>
