<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import Navbar from '@/components/layout/Navbar.vue'
import ToastContainer from '@/components/ui/ToastContainer.vue'
import { Tractor } from 'lucide-vue-next'
import { useCompanionStore } from '@/stores/companion'

const route = useRoute()
const isAuthRoute = computed(() => route.path === '/auth')
const companionStore = useCompanionStore()

onMounted(() => {
  companionStore.initTauriListeners()
})
</script>

<template>
  <div class="min-h-screen bg-background text-foreground flex flex-col font-sans">
    <!-- Navbar (Hidden on auth if preferred, or present with minimal look) -->
    <Navbar v-if="!isAuthRoute" />

    <!-- Main Content -->
    <main class="flex-1">
      <router-view />
    </main>

    <!-- Cockpit Technical Footer -->
    <footer v-if="!isAuthRoute" class="bg-surface border-t border-surface-border py-4 text-xs font-mono text-muted select-none">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col sm:flex-row items-center justify-between gap-3">
        <div class="flex items-center gap-2">
          <Tractor class="w-4 h-4 text-primary" />
          <span class="font-bold text-foreground">FARMLINK FS25</span>
          <span>&bull;</span>
          <span>Protocole Modpack v2.4</span>
        </div>

        <div class="flex items-center gap-4 text-[11px]">
          <span>GIANTS Software Compatible</span>
          <span>&bull;</span>
          <span>Backend Go (REST/S3) &bull; Compagnon Rust (IPC)</span>
        </div>
      </div>
    </footer>

    <!-- Global Industrial Toasts -->
    <ToastContainer />
  </div>
</template>
