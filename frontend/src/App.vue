<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import Navbar from '@/components/layout/Navbar.vue'
import ToastContainer from '@/components/ui/ToastContainer.vue'
import { useCompanionStore } from '@/stores/companion'

const route = useRoute()
const isAuthRoute = computed(() => route.path === '/auth')
const companionStore = useCompanionStore()

onMounted(() => {
  companionStore.initTauriListeners()
})
</script>

<template>
  <div class="min-h-screen bg-background text-foreground flex flex-col font-sans transition-colors duration-150">
    <!-- Navbar (Hidden on auth) -->
    <Navbar v-if="!isAuthRoute" />

    <!-- Main Content -->
    <main class="flex-1">
      <router-view />
    </main>

    <!-- Clean Minimal Footer -->
    <footer v-if="!isAuthRoute" class="bg-surface border-t border-surface-border py-5 text-xs text-muted select-none">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col sm:flex-row items-center justify-between gap-4">
        <div class="flex items-center gap-3">
          <img src="/logo_navbar.png" alt="FarmLink" class="h-6 w-auto object-contain shrink-0" />
          <span class="font-display text-xs font-bold uppercase tracking-wider text-foreground">
            FarmLink FS25
          </span>
        </div>

        <p class="text-xs text-muted font-sans">
          © 2026 FarmLink
        </p>
      </div>
    </footer>

    <!-- Global Toasts -->
    <ToastContainer />
  </div>
</template>
