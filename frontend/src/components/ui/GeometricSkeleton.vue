<script setup lang="ts">
import { cn } from '@/lib/utils'

interface Props {
  className?: string
  width?: string
  height?: string
}

withDefaults(defineProps<Props>(), {
  className: '',
  width: 'w-full',
  height: 'h-4',
})
</script>

<template>
  <div
    :class="
      cn(
        'relative bg-surface-raised border border-surface-border overflow-hidden animate-industrial-blink rounded-none',
        width,
        height,
        className
      )
    "
  >
    <!-- Geometric hatch overlay -->
    <div
      class="absolute inset-0 opacity-15"
      style="background: repeating-linear-gradient(45deg, #438E33, #438E33 2px, transparent 2px, transparent 8px);"
    ></div>
  </div>
</template>
