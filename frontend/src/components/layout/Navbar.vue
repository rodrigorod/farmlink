<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useCompanionStore } from '@/stores/companion'
import {
  Tractor,
  Layers,
  Settings,
  Shield,
  Radio,
  LogOut,
  ChevronDown,
} from 'lucide-vue-next'
import CompanionModal from '@/components/ui/CompanionModal.vue'

const authStore = useAuthStore()
const companionStore = useCompanionStore()
const router = useRouter()
const route = useRoute()

const isCompanionModalOpen = ref(false)
const isUserMenuOpen = ref(false)

function toggleRole() {
  const newRole = authStore.user.role === 'admin' ? 'player' : 'admin'
  authStore.setRole(newRole)
}

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
            App Compagnon Rust
          </router-link>
        </nav>
      </div>

      <!-- Right: Companion Status LED + Role Switcher + User Profile -->
      <div class="flex items-center gap-3">
        <!-- Rust Companion App Permanent Indicator -->
        <button
          type="button"
          @click="isCompanionModalOpen = true"
          :class="[
            'px-2.5 py-1.5 rounded-sm border text-xs font-mono flex items-center gap-2 transition-all',
            companionStore.state.isConnected
              ? 'bg-emerald-950/40 border-emerald-800/80 text-emerald-400 hover:bg-emerald-950/70'
              : 'bg-red-950/40 border-red-800/80 text-red-400 hover:bg-red-950/70',
          ]"
          title="Cliquez pour afficher les détails du pont Rust compagnon"
        >
          <span class="relative flex h-2 w-2">
            <span
              v-if="companionStore.state.isConnected"
              class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"
            ></span>
            <span
              :class="[
                'relative inline-flex rounded-full h-2 w-2',
                companionStore.state.isConnected ? 'bg-primary' : 'bg-red-500',
              ]"
            ></span>
          </span>
          <Radio class="w-3.5 h-3.5" />
          <span class="hidden sm:inline font-semibold tracking-wide">
            {{ companionStore.state.isConnected ? 'RUST CLIENT : ACTIF' : 'RUST CLIENT : HORS-LIGNE' }}
          </span>
        </button>

        <!-- Role Switcher (Admin vs Player) for fast evaluation & testing -->
        <button
          type="button"
          @click="toggleRole"
          :class="[
            'px-2 py-1 text-xs font-mono font-bold rounded-sm border uppercase tracking-wider transition-colors flex items-center gap-1.5',
            authStore.isAdmin
              ? 'bg-purple-950/60 text-purple-300 border-purple-800/80 hover:bg-purple-900/60'
              : 'bg-surface-raised text-muted border-surface-border hover:bg-surface-hover hover:text-foreground',
          ]"
          title="Cliquez pour basculer rapidement entre Joueur et Administrateur"
        >
          <Shield class="w-3.5 h-3.5 text-purple-400" />
          <span class="hidden sm:inline text-[11px]">Rôle:</span>
          <span>{{ authStore.user.role }}</span>
        </button>

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
            <span class="hidden lg:inline text-xs font-medium text-foreground">
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

  <!-- Modal for Rust companion status -->
  <CompanionModal
    :is-open="isCompanionModalOpen"
    @close="isCompanionModalOpen = false"
  />
</template>
