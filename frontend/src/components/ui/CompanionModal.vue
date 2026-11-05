<script setup lang="ts">
import { useCompanionStore } from '@/stores/companion'
import {
  HardDrive,
  Cpu,
  RefreshCw,
  Folder,
  Radio,
  ExternalLink,
  X,
  AlertTriangle,
} from 'lucide-vue-next'
import Button from './Button.vue'
import Badge from './Badge.vue'

defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const companionStore = useCompanionStore()
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-xs"
  >
    <div
      class="industrial-panel bg-surface max-w-lg w-full border border-surface-border shadow-2xl p-6 relative"
    >
      <!-- Header -->
      <div class="flex items-center justify-between pb-4 border-b border-surface-border">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-surface-raised border border-surface-border rounded-sm">
            <Radio
              :class="[
                'w-5 h-5',
                companionStore.state.isConnected ? 'text-primary' : 'text-accent',
              ]"
            />
          </div>
          <div>
            <h3 class="text-sm font-bold uppercase tracking-wider text-foreground">
              Pont App Compagnon Rust (IPC / WS)
            </h3>
            <p class="text-xs text-muted font-mono">
              ws://127.0.0.1:45823 &bull; Statut du connecteur natif
            </p>
          </div>
        </div>
        <button
          type="button"
          @click="emit('close')"
          class="text-muted hover:text-foreground p-1"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Body / Status Specs -->
      <div class="py-5 flex flex-col gap-4 text-xs">
        <div class="p-3 bg-surface-raised border border-surface-border flex items-center justify-between">
          <span class="text-muted uppercase tracking-wider font-semibold">État de la liaison :</span>
          <div class="flex items-center gap-2">
            <Badge :variant="companionStore.state.isConnected ? 'synced' : 'missing'">
              <span
                :class="[
                  'w-1.5 h-1.5 rounded-full inline-block mr-1',
                  companionStore.state.isConnected ? 'bg-primary' : 'bg-destructive',
                ]"
              ></span>
              {{ companionStore.state.isConnected ? 'Connecté / IPC Actif' : 'Non détecté' }}
            </Badge>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div class="p-3 bg-surface-raised border border-surface-border">
            <div class="flex items-center gap-1.5 text-muted mb-1 font-semibold uppercase">
              <Cpu class="w-3.5 h-3.5" />
              Version Client
            </div>
            <div class="font-mono text-sm font-bold text-foreground">
              {{ companionStore.state.version }}
            </div>
          </div>

          <div class="p-3 bg-surface-raised border border-surface-border">
            <div class="flex items-center gap-1.5 text-muted mb-1 font-semibold uppercase">
              <HardDrive class="w-3.5 h-3.5" />
              OS Détecté
            </div>
            <div class="font-mono text-sm uppercase text-foreground">
              {{ companionStore.state.os }} x64
            </div>
          </div>
        </div>

        <div class="p-3 bg-surface-raised border border-surface-border">
          <div class="flex items-center gap-1.5 text-muted mb-1 font-semibold uppercase">
            <Folder class="w-3.5 h-3.5" />
            Répertoire FS25 local configuré
          </div>
          <div class="font-mono text-[11px] text-primary truncate bg-surface p-2 border border-surface-border mt-1">
            {{ companionStore.state.modsPath }}
          </div>
        </div>

        <!-- Progress bar if syncing -->
        <div v-if="companionStore.state.syncInProgress" class="p-3 bg-surface border border-primary/50">
          <div class="flex items-center justify-between text-xs mb-1">
            <span class="text-primary font-semibold flex items-center gap-1.5">
              <RefreshCw class="w-3.5 h-3.5 animate-spin" />
              Synchronisation en cours...
            </span>
            <span class="font-mono font-bold text-primary">{{ companionStore.state.syncProgress }}%</span>
          </div>
          <div class="w-full h-2 bg-surface-raised border border-surface-border overflow-hidden">
            <div
              class="h-full bg-primary transition-all duration-300"
              :style="{ width: `${companionStore.state.syncProgress}%` }"
            ></div>
          </div>
          <div class="text-[11px] text-muted font-mono mt-1 truncate">
            Transfert de : {{ companionStore.state.currentSyncMod }}
          </div>
        </div>

        <div
          v-if="!companionStore.state.isConnected"
          class="p-3 bg-amber-950/40 border border-amber-800/60 text-amber-200 flex items-start gap-2 text-xs"
        >
          <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5 text-amber-400" />
          <span>
            L'application cliente Rust ne répond pas sur <code class="font-mono bg-black/40 px-1 py-0.5">127.0.0.1:45823</code>. Assurez-vous qu'elle est lancée sur votre machine pour synchroniser vos mods d'un simple clic.
          </span>
        </div>
      </div>

      <!-- Actions -->
      <div class="pt-4 border-t border-surface-border flex flex-wrap items-center justify-between gap-3">
        <div class="flex items-center gap-2">
          <Button
            size="sm"
            :variant="companionStore.state.isConnected ? 'outline' : 'primary'"
            @click="companionStore.toggleConnection"
          >
            {{ companionStore.state.isConnected ? 'Simuler Déconnexion' : 'Simuler Connexion' }}
          </Button>

          <Button
            size="sm"
            variant="secondary"
            @click="companionStore.tryRealWebSocketConnect"
            :disabled="companionStore.state.isConnecting"
          >
            <RefreshCw :class="['w-3.5 h-3.5', companionStore.state.isConnecting ? 'animate-spin' : '']" />
            Tester WS Local
          </Button>
        </div>

        <router-link to="/settings" @click="emit('close')">
          <Button size="sm" variant="ghost" class="text-xs">
            Télécharger le client
            <ExternalLink class="w-3.5 h-3.5 ml-1" />
          </Button>
        </router-link>
      </div>
    </div>
  </div>
</template>
