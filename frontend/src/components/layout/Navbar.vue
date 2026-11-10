<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useCompanionStore } from '@/stores/companion'
import { currentTheme, toggleTheme } from '@/lib/theme'
import {
  Layers,
  Settings,
  LogOut,
  ChevronDown,
  RefreshCw,
  Sun,
  Moon,
} from 'lucide-vue-next'

const authStore = useAuthStore()
const companionStore = useCompanionStore()
const router = useRouter()
const route = useRoute()

const isUserMenuOpen = ref(false)

function handleLogout() {
  authStore.logout()
  router.push('/auth')
}
</script>

<template>
  <header class="bg-surface border-b border-surface-border sticky top-0 z-40 select-none transition-colors">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
      
      <!-- Left: Logo & Swiss Brand -->
      <div class="flex items-center gap-6">
        <router-link to="/dashboard" class="flex items-center gap-3 group focus:outline-none">
          <img
            src="/logo_navbar.png"
            alt="FarmLink Logo"
            class="h-8 w-auto object-contain transition-transform group-hover:scale-105 shrink-0"
          />
          <div class="flex items-center gap-2">
            <span class="text-base font-black tracking-tight uppercase text-foreground font-display">
              FarmLink
            </span>
            <span class="px-2 py-0.5 bg-primary/10 text-primary font-mono text-[10px] font-bold rounded-full border border-primary/25">
              FS25
            </span>
          </div>
        </router-link>

        <!-- Swiss Navigation Pills -->
        <nav class="hidden md:flex items-center gap-1.5 border-l border-surface-border pl-6 ml-2">
          <router-link
            to="/dashboard"
            :class="[
              'px-3.5 py-1.5 text-xs font-bold uppercase tracking-wider rounded-full transition-all flex items-center gap-2',
              route.path.startsWith('/dashboard') || route.path.startsWith('/servers')
                ? 'bg-brand-dark text-white dark:bg-white dark:text-brand-dark shadow-swiss-sm'
                : 'text-muted hover:text-foreground hover:bg-surface-hover',
            ]"
          >
            <Layers class="w-3.5 h-3.5" />
            Serveurs
          </router-link>

          <router-link
            to="/settings"
            :class="[
              'px-3.5 py-1.5 text-xs font-bold uppercase tracking-wider rounded-full transition-all flex items-center gap-2',
              route.path === '/settings'
                ? 'bg-brand-dark text-white dark:bg-white dark:text-brand-dark shadow-swiss-sm'
                : 'text-muted hover:text-foreground hover:bg-surface-hover',
            ]"
          >
            <Settings class="w-3.5 h-3.5" />
            Paramètres
          </router-link>
        </nav>
      </div>

      <!-- Right: Sync status, Theme switch & User Menu -->
      <div class="flex items-center gap-2.5 sm:gap-3">
        <!-- Live Sync Progress Indicator if syncing -->
        <div
          v-if="companionStore.state.syncInProgress"
          class="px-3 py-1 rounded-full bg-primary/10 border border-primary/30 text-primary text-xs font-mono flex items-center gap-2"
        >
          <RefreshCw class="w-3.5 h-3.5 animate-spin" />
          <span class="font-bold">{{ companionStore.state.syncProgress }}%</span>
          <span v-if="companionStore.state.currentSyncMod" class="hidden xl:inline text-[11px] text-muted truncate max-w-[120px]">
            {{ companionStore.state.currentSyncMod }}
          </span>
        </div>

        <!-- Theme Toggle Button (Light/Dark Swiss Mode) -->
        <button
          type="button"
          @click="toggleTheme"
          :title="currentTheme === 'dark' ? 'Activer le mode clair (Swiss Canvas)' : 'Activer le mode sombre'"
          class="w-9 h-9 rounded-full bg-surface-raised border border-surface-border flex items-center justify-center text-muted hover:text-foreground hover:bg-surface-hover transition-colors focus:outline-none"
        >
          <Sun v-if="currentTheme === 'dark'" class="w-4 h-4 text-amber-400" />
          <Moon v-else class="w-4 h-4 text-slate-700" />
        </button>

        <!-- User Menu Dropdown -->
        <div class="relative">
          <button
            type="button"
            @click="isUserMenuOpen = !isUserMenuOpen"
            class="flex items-center gap-2 px-2.5 py-1.5 bg-surface-raised border border-surface-border rounded-full hover:border-foreground/30 transition-all focus:outline-none"
          >
            <div class="w-6 h-6 bg-brand-dark text-white dark:bg-white dark:text-brand-dark flex items-center justify-center font-bold text-xs rounded-full">
              {{ authStore.user.username.charAt(0).toUpperCase() }}
            </div>
            <span class="hidden sm:inline text-xs font-bold uppercase tracking-wider text-foreground font-display">
              {{ authStore.user.username }}
            </span>
            <ChevronDown class="w-3.5 h-3.5 text-muted" />
          </button>

          <!-- Dropdown popup -->
          <div
            v-if="isUserMenuOpen"
            @click.away="isUserMenuOpen = false"
            class="absolute right-0 mt-2 w-52 bg-surface border border-surface-border shadow-swiss-lg py-1.5 z-50 rounded-xl"
          >
            <div class="px-3.5 py-2.5 border-b border-surface-border text-xs">
              <div class="font-bold uppercase tracking-wider text-foreground font-display">{{ authStore.user.username }}</div>
              <div class="text-[11px] text-muted font-mono mt-0.5 flex items-center gap-1.5">
                <span class="w-1.5 h-1.5 rounded-full bg-primary inline-block"></span>
                <span>{{ authStore.user.role === 'admin' ? 'Administrateur Serveur' : 'Joueur Co-op' }}</span>
              </div>
            </div>

            <div class="p-1">
              <router-link
                to="/settings"
                @click="isUserMenuOpen = false"
                class="flex items-center gap-2 px-3 py-2 text-xs font-medium text-foreground hover:bg-surface-hover rounded-md transition-colors"
              >
                <Settings class="w-3.5 h-3.5 text-muted" />
                Paramètres FS25
              </router-link>

              <button
                type="button"
                @click="handleLogout"
                class="w-full flex items-center gap-2 px-3 py-2 text-xs font-medium text-destructive hover:bg-destructive/10 rounded-md transition-colors text-left"
              >
                <LogOut class="w-3.5 h-3.5" />
                Se déconnecter
              </button>
            </div>
          </div>
        </div>
      </div>

    </div>
  </header>
</template>
