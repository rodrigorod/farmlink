<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useCompanionStore } from '@/stores/companion'
import {
  Tractor,
  Layers,
  Settings,
  LogOut,
  ChevronDown,
  RefreshCw,
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
  <header class="bg-surface border-b border-surface-border sticky top-0 z-40 select-none">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
      
      <!-- Left: Logo & Cockpit Brand -->
      <div class="flex items-center gap-6">
        <router-link to="/dashboard" class="flex items-center gap-3 group">
          <div class="w-10 h-10 rounded-sm bg-primary/10 border border-primary/40 flex items-center justify-center text-primary group-hover:bg-primary group-hover:text-white transition-all">
            <Tractor class="w-6 h-6" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <span class="text-sm font-black tracking-wider uppercase text-foreground">
                FarmLink
              </span>
              <span class="px-1.5 py-0.2 bg-primary/20 text-primary font-mono text-[10px] font-bold rounded-sm border border-primary/30">
                FS25
              </span>
            </div>
            <p class="text-[10px] text-muted tracking-tight font-mono">
              Industrial Modpack Manager
            </p>
          </div>
        </router-link>

        <!-- Navigation Links -->
        <nav class="hidden md:flex items-center gap-1 border-l border-surface-border pl-6 ml-2">
          <router-link
            to="/dashboard"
            :class="[
              'px-3 py-1.5 text-xs font-semibold uppercase tracking-wider rounded-sm transition-colors flex items-center gap-2',
              route.path.startsWith('/dashboard') || route.path.startsWith('/servers')
                ? 'bg-surface-raised text-primary border border-surface-border'
                : 'text-muted hover:text-foreground hover:bg-surface-hover',
            ]"
          >
            <Layers class="w-3.5 h-3.5" />
            Serveurs
          </router-link>

          <router-link
            to="/settings"
            :class="[
              'px-3 py-1.5 text-xs font-semibold uppercase tracking-wider rounded-sm transition-colors flex items-center gap-2',
              route.path === '/settings'
                ? 'bg-surface-raised text-primary border border-surface-border'
                : 'text-muted hover:text-foreground hover:bg-surface-hover',
            ]"
          >
            <Settings class="w-3.5 h-3.5" />
            Paramètres
          </router-link>
        </nav>
      </div>

      <!-- Right: Sync status & User Menu -->
      <div class="flex items-center gap-3">
        <!-- Live Sync Progress Indicator if syncing -->
        <div
          v-if="companionStore.state.syncInProgress"
          class="px-2.5 py-1 rounded-sm bg-primary/10 border border-primary/30 text-primary text-xs font-mono flex items-center gap-2"
        >
          <RefreshCw class="w-3.5 h-3.5 animate-spin" />
          <span class="font-bold">{{ companionStore.state.syncProgress }}%</span>
          <span v-if="companionStore.state.currentSyncMod" class="hidden xl:inline text-[11px] text-muted truncate max-w-[120px]">
            {{ companionStore.state.currentSyncMod }}
          </span>
        </div>

        <!-- User Menu Dropdown -->
        <div class="relative">
          <button
            type="button"
            @click="isUserMenuOpen = !isUserMenuOpen"
            class="flex items-center gap-2 p-1.5 bg-surface-raised border border-surface-border rounded-sm hover:border-gray-500 transition-colors"
          >
            <div class="w-7 h-7 bg-surface border border-surface-border flex items-center justify-center text-primary font-bold text-xs rounded-sm">
              {{ authStore.user.username.charAt(0) }}
            </div>
            <span class="hidden sm:inline text-xs font-medium text-foreground">
              {{ authStore.user.username }}
            </span>
            <ChevronDown class="w-3.5 h-3.5 text-muted" />
          </button>

          <!-- Dropdown popup -->
          <div
            v-if="isUserMenuOpen"
            @click.away="isUserMenuOpen = false"
            class="absolute right-0 mt-2 w-48 bg-surface border border-surface-border shadow-xl py-1 z-50 rounded-sm"
          >
            <div class="px-3 py-2 border-b border-surface-border text-xs">
              <div class="font-bold text-foreground">{{ authStore.user.username }}</div>
              <div class="text-[11px] text-muted truncate">{{ authStore.user.email }}</div>
            </div>

            <router-link
              to="/settings"
              @click="isUserMenuOpen = false"
              class="flex items-center gap-2 px-3 py-2 text-xs text-muted hover:text-foreground hover:bg-surface-raised"
            >
              <Settings class="w-3.5 h-3.5" />
              Configuration
            </router-link>

            <button
              type="button"
              @click="handleLogout"
              class="w-full flex items-center gap-2 px-3 py-2 text-xs text-destructive hover:bg-destructive/10 text-left"
            >
              <LogOut class="w-3.5 h-3.5" />
              Déconnexion
            </button>
          </div>
        </div>
      </div>
    </div>
  </header>
</template>
