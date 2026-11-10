<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  variant?: 'primary' | 'secondary' | 'accent' | 'destructive' | 'outline' | 'ghost' | 'dark'
  size?: 'sm' | 'md' | 'lg' | 'pill' | 'icon' | 'icon-circle'
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
      return 'bg-primary text-white hover:bg-primary-hover active:translate-y-px shadow-swiss-sm'
    case 'dark':
      return 'bg-brand-dark text-white hover:bg-black/90 active:translate-y-px shadow-swiss-sm'
    case 'accent':
      return 'bg-primary text-white hover:bg-primary-hover active:translate-y-px'
    case 'destructive':
      return 'bg-destructive text-white hover:bg-red-700 active:translate-y-px'
    case 'secondary':
      return 'bg-surface-raised text-foreground hover:bg-surface-hover border border-surface-border'
    case 'outline':
      return 'border border-surface-border bg-surface text-foreground hover:border-foreground/40 hover:bg-surface-hover'
    case 'ghost':
      return 'text-muted hover:text-foreground hover:bg-surface-hover'
    default:
      return 'bg-primary text-white'
  }
})

const sizeClasses = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'px-2.5 py-1 text-xs font-semibold uppercase tracking-wider rounded-md'
    case 'lg':
      return 'px-5 py-2.5 text-sm font-bold uppercase tracking-wider rounded-md'
    case 'pill':
      return 'px-4 py-1.5 text-xs font-semibold uppercase tracking-wider rounded-full border border-surface-border'
    case 'icon':
      return 'p-2 rounded-md'
    case 'icon-circle':
      return 'w-9 h-9 rounded-full p-0 flex items-center justify-center border border-surface-border'
    case 'md':
    default:
      return 'px-4 py-2 text-xs font-bold uppercase tracking-wider rounded-md'
  }
})
</script>

<template>
  <button
    :type="type"
    :disabled="disabled"
    :class="
      cn(
        'inline-flex items-center justify-center gap-2 font-display transition-all select-none focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background disabled:opacity-40 disabled:pointer-events-none',
        variantClasses,
        sizeClasses,
        className
      )
    "
  >
    <slot />
  </button>
</template>
