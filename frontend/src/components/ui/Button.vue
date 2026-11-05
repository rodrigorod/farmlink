<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  variant?: 'primary' | 'secondary' | 'accent' | 'destructive' | 'outline' | 'ghost'
  size?: 'sm' | 'md' | 'lg' | 'icon'
  disabled?: boolean
  type?: 'button' | 'submit' | 'reset'
  className?: string
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  type: 'button',
  className: '',
})

const variantClasses = computed(() => {
  switch (props.variant) {
    case 'primary':
      return 'bg-primary text-white hover:bg-primary-hover active:translate-y-px border-b-2 border-emerald-800'
    case 'accent':
      return 'bg-accent text-white hover:bg-accent-hover active:translate-y-px border-b-2 border-amber-800'
    case 'destructive':
      return 'bg-destructive text-white hover:bg-red-700 active:translate-y-px border-b-2 border-red-900'
    case 'secondary':
      return 'bg-surface-raised text-foreground hover:bg-surface-hover border border-surface-border'
    case 'outline':
      return 'border border-surface-border text-foreground hover:bg-surface-hover hover:border-gray-500'
    case 'ghost':
      return 'text-muted hover:text-foreground hover:bg-surface-hover'
    default:
      return 'bg-primary text-white'
  }
})

const sizeClasses = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'px-2.5 py-1 text-xs font-medium'
    case 'lg':
      return 'px-5 py-2.5 text-base font-semibold'
    case 'icon':
      return 'p-2'
    case 'md':
    default:
      return 'px-4 py-2 text-sm font-medium'
  }
})
</script>

<template>
  <button
    :type="type"
    :disabled="disabled"
    :class="
      cn(
        'inline-flex items-center justify-center gap-2 rounded-sm tracking-wide transition-all select-none focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background disabled:opacity-50 disabled:pointer-events-none',
        variantClasses,
        sizeClasses,
        className
      )
    "
  >
    <slot />
  </button>
</template>
