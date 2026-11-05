<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useServersStore } from '@/stores/servers'
import { useCompanionStore } from '@/stores/companion'
import { useToastStore } from '@/stores/toast'
import { useAuthStore } from '@/stores/auth'
import { formatBytes, formatDate } from '@/lib/utils'
import {
  Server as ServerIcon,
  Search,
  Users,
  HardDrive,
  Copy,
  Shield,
  Layers,
  Play,
  RotateCw,
  CheckCircle2,
  AlertTriangle,
  XCircle,
} from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
import GeometricSkeleton from '@/components/ui/GeometricSkeleton.vue'

const serversStore = useServersStore()
const companionStore = useCompanionStore()
const toastStore = useToastStore()
const authStore = useAuthStore()

const isLoading = ref(true)
const searchQuery = ref('')
const selectedTld = ref('all')

const syncedCount = computed(() => serversStore.servers.filter(s => s.status === 'synced').length)
const updateRequiredCount = computed(() => serversStore.servers.filter(s => s.status === 'update_required').length)
const errorCount = computed(() => serversStore.servers.filter(s => s.status === 'error_desync').length)

const filteredServers = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  return serversStore.servers.filter((s) => {
    const addr = s.address.toLowerCase()
    const matchesSearch = !q || s.name.toLowerCase().includes(q) || addr.includes(q)
    const matchesTld = selectedTld.value === 'all' || addr.includes(selectedTld.value.toLowerCase())
    return matchesSearch && matchesTld
  })
})

onMounted(async () => {
  isLoading.value = true
  await serversStore.fetchServers()
  isLoading.value = false
})

async function simulateRefresh() {
  isLoading.value = true
  await serversStore.fetchServers()
  isLoading.value = false
  toastStore.info('Actualisation', 'État des serveurs FS25 synchronisé avec la base de données.')
}

function copyAddress(address: string) {
  navigator.clipboard.writeText(address)
  toastStore.success('Adresse copiée', `L'adresse IP "${address}" a été copiée dans le presse-papier.`)
}

function joinServer(server: any) {
  if (companionStore.state.isConnected) {
    toastStore.success(
      'Lancement via Rust Compagnon',
      `Ouverture de Farming Simulator 25 et connexion directe à "${server.name}" (${server.address})...`
    )
    // Trigger native companion protocol safely without navigating the current page
    const iframe = document.createElement('iframe')
    iframe.style.display = 'none'
    iframe.src = `farmlink://join?server=${server.id}&addr=${encodeURIComponent(server.address)}`
    document.body.appendChild(iframe)
    setTimeout(() => iframe.remove(), 2000)
  } else {
    copyAddress(server.address)
    toastStore.warning(
      'App Compagnon non détectée',
      `L'adresse ${server.address} a été copiée. Pour une connexion automatique 1-clic, démarrez l'app Rust.`
    )
  }
}
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
    
    <!-- Hero / Cockpit Header -->
    <div class="industrial-panel p-6 bg-surface flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 bg-primary/20 border border-primary/40 rounded-sm flex items-center justify-center text-primary">
            <ServerIcon class="w-5 h-5" />
          </div>
          <h1 class="text-xl font-black uppercase tracking-wider text-foreground">
            Tableau de Bord des Serveurs FS25
          </h1>
        </div>
        <p class="text-xs text-muted mt-1 font-mono">
          Surveillance en temps réel des fermes multijoueurs et synchronisation des modpacks
        </p>
      </div>

      <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-2">
        <div class="relative w-full sm:w-64">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Rechercher par nom, IP..."
            class="w-full bg-surface-raised border border-surface-border rounded-sm px-3 py-1.5 pl-8 text-xs text-foreground focus:outline-none focus:border-primary font-mono placeholder:text-muted/60"
          />
          <Search class="w-3.5 h-3.5 absolute left-2.5 top-2.5 text-muted pointer-events-none" />
        </div>

        <select
          v-model="selectedTld"
          class="bg-surface-raised border border-surface-border rounded-sm px-2.5 py-1.5 text-xs text-foreground focus:outline-none focus:border-primary font-mono cursor-pointer"
        >
          <option value="all">Zone : Toutes</option>
          <option value=".eu">🇪🇺 Europe (.eu)</option>
          <option value=".fr">🇫🇷 France (.fr)</option>
          <option value=".de">🇩🇪 Allemagne (.de)</option>
          <option value=".com">🌐 Inter (.com)</option>
          <option value=".us">🇺🇸 USA (.us)</option>
        </select>

        <Button size="sm" variant="outline" @click="simulateRefresh" :disabled="isLoading">
          <RotateCw :class="['w-3.5 h-3.5', isLoading ? 'animate-spin' : '']" />
          Actualiser
        </Button>
      </div>
    </div>

    <!-- Quick Legend / Metric Bar -->
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
      <div class="industrial-panel p-4 bg-surface border-l-4 border-l-primary flex items-center justify-between">
        <div>
          <div class="text-[10px] text-muted uppercase font-bold tracking-wider">🟢 Prêt à jouer</div>
          <div class="text-lg font-bold font-mono text-foreground mt-0.5">
            {{ syncedCount }} Serveur{{ syncedCount > 1 ? 's' : '' }}
          </div>
        </div>
        <CheckCircle2 class="w-6 h-6 text-primary opacity-80" />
      </div>

      <div class="industrial-panel p-4 bg-surface border-l-4 border-l-accent flex items-center justify-between">
        <div>
          <div class="text-[10px] text-muted uppercase font-bold tracking-wider">🟠 Mise à jour requise</div>
          <div class="text-lg font-bold font-mono text-foreground mt-0.5">
            {{ updateRequiredCount }} Serveur{{ updateRequiredCount > 1 ? 's' : '' }}
          </div>
        </div>
        <AlertTriangle class="w-6 h-6 text-accent opacity-80" />
      </div>

      <div class="industrial-panel p-4 bg-surface border-l-4 border-l-destructive flex items-center justify-between">
        <div>
          <div class="text-[10px] text-muted uppercase font-bold tracking-wider">🔴 Erreur / Désynchronisé</div>
          <div class="text-lg font-bold font-mono text-foreground mt-0.5">
            {{ errorCount }} Serveur{{ errorCount > 1 ? 's' : '' }}
          </div>
        </div>
        <XCircle class="w-6 h-6 text-destructive opacity-80" />
      </div>
    </div>

    <!-- Loading Skeletons or Server List -->
    <div v-if="isLoading" class="space-y-4">
      <div v-for="i in 3" :key="i" class="industrial-panel p-6 bg-surface space-y-4">
        <div class="flex justify-between items-center">
          <GeometricSkeleton width="w-64" height="h-6" />
          <GeometricSkeleton width="w-32" height="h-6" />
        </div>
        <div class="grid grid-cols-4 gap-4">
          <GeometricSkeleton height="h-10" />
          <GeometricSkeleton height="h-10" />
          <GeometricSkeleton height="h-10" />
          <GeometricSkeleton height="h-10" />
        </div>
      </div>
    </div>

    <!-- Real Server Cards Grid -->
    <div v-else-if="filteredServers.length > 0" class="space-y-4">
      <div
        v-for="server in filteredServers"
        :key="server.id"
        class="industrial-panel p-6 bg-surface hover:border-gray-500 transition-all space-y-5"
      >
        <!-- Top Row: Server Name, Map, Status Badge -->
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-4 border-b border-surface-border">
          <div>
            <div class="flex items-center gap-3">
              <h2 class="text-base font-bold uppercase tracking-wider text-foreground">
                {{ server.name }}
              </h2>
              <span class="text-xs font-mono px-2 py-0.5 bg-surface-raised border border-surface-border text-muted">
                {{ server.mapName }}
              </span>
              <span
                v-if="server.isOwner"
                class="px-2 py-0.5 bg-purple-950/60 border border-purple-800 text-purple-300 text-[10px] uppercase font-bold tracking-wider"
              >
                Propriétaire
              </span>
            </div>
            <div class="flex items-center gap-2 mt-1 text-xs text-muted font-mono">
              <span>{{ server.address }}</span>
              <button
                type="button"
                @click="copyAddress(server.address)"
                class="hover:text-foreground p-0.5"
                title="Copier l'adresse"
              >
                <Copy class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>

          <!-- Quick Status Indicator -->
          <div>
            <Badge v-if="server.status === 'synced'" variant="synced">
              🟢 Synchronisé / Prêt à jouer
            </Badge>
            <Badge v-else-if="server.status === 'update_required'" variant="outdated">
              🟠 Mise à jour requise
            </Badge>
            <Badge v-else variant="error">
              🔴 Désynchronisé / Conflit
            </Badge>
          </div>
        </div>

        <!-- Metrics Grid -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
          <div class="p-3 bg-surface-raised border border-surface-border">
            <div class="text-muted flex items-center gap-1.5 font-semibold uppercase text-[10px]">
              <Users class="w-3.5 h-3.5 text-primary" />
              Joueurs en ligne
            </div>
            <div class="text-sm font-mono font-bold text-foreground mt-1">
              {{ server.currentPlayers }} / {{ server.maxPlayers }}
            </div>
          </div>

          <div class="p-3 bg-surface-raised border border-surface-border">
            <div class="text-muted flex items-center gap-1.5 font-semibold uppercase text-[10px]">
              <Layers class="w-3.5 h-3.5 text-primary" />
              Version Modpack
            </div>
            <div class="text-sm font-mono font-bold text-primary mt-1">
              {{ server.modpackVersion }}
            </div>
          </div>

          <div class="p-3 bg-surface-raised border border-surface-border">
            <div class="text-muted flex items-center gap-1.5 font-semibold uppercase text-[10px]">
              <HardDrive class="w-3.5 h-3.5 text-primary" />
              Volume Mods
            </div>
            <div class="text-sm font-mono font-bold text-foreground mt-1">
              {{ server.modCount }} mods ({{ formatBytes(server.totalSizeBytes) }})
            </div>
          </div>

          <div class="p-3 bg-surface-raised border border-surface-border">
            <div class="text-muted flex items-center gap-1.5 font-semibold uppercase text-[10px]">
              <RotateCw class="w-3.5 h-3.5 text-primary" />
              Dernier Manifeste
            </div>
            <div class="text-xs font-mono text-muted mt-1 truncate">
              {{ formatDate(server.lastUpdated) }}
            </div>
          </div>
        </div>

        <!-- Bottom Action Bar -->
        <div class="pt-3 border-t border-surface-border flex flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <!-- View Modpack Player View -->
            <router-link :to="`/servers/${server.id}`">
              <Button size="sm" variant="secondary">
                <Layers class="w-4 h-4 text-primary" />
                Détail du Modpack ({{ server.modCount }} mods)
              </Button>
            </router-link>

            <!-- Admin Button if user is Admin or Server Owner -->
            <router-link
              v-if="authStore.isAdmin || server.isOwner"
              :to="`/servers/${server.id}/admin`"
            >
              <Button size="sm" variant="outline" class="border-purple-800/80 text-purple-300 hover:bg-purple-950/40">
                <Shield class="w-3.5 h-3.5" />
                Gérer le Manifeste (Admin)
              </Button>
            </router-link>
          </div>

          <!-- Quick Action: Rejoindre le serveur -->
          <Button
            size="sm"
            :variant="server.status === 'synced' ? 'primary' : 'accent'"
            @click="joinServer(server)"
          >
            <Play class="w-4 h-4" />
            Rejoindre le serveur
          </Button>
        </div>
      </div>
    </div>

    <!-- Empty Search Results -->
    <div v-else class="industrial-panel p-8 bg-surface text-center">
      <p class="text-sm text-muted font-mono">
        Aucun serveur trouvé
        <span v-if="searchQuery"> pour la recherche "{{ searchQuery }}"</span>
        <span v-if="selectedTld !== 'all'"> avec l'extension "{{ selectedTld }}"</span>
      </p>
    </div>
  </div>
</template>
