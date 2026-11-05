<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useServersStore } from '@/stores/servers'
import { useCompanionStore } from '@/stores/companion'
import { useToastStore } from '@/stores/toast'
import { formatBytes } from '@/lib/utils'
import {
  DownloadCloud,
  RefreshCw,
  Search,
  Shield,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  FileArchive,
  ArrowLeft,
  Copy,
} from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'

const route = useRoute()
const serversStore = useServersStore()
const companionStore = useCompanionStore()
const toastStore = useToastStore()

const serverId = computed(() => route.params.id as string)
const server = serversStore.getServerById(serverId.value)
const mods = serversStore.getModsForServer(serverId.value)

onMounted(async () => {
  await serversStore.fetchServers()
  await serversStore.fetchManifest(serverId.value)
})

const searchQuery = ref('')
const selectedCategory = ref<string>('all')

const filteredMods = computed(() => {
  return mods.value.filter((mod) => {
    const matchesSearch =
      mod.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      mod.fileName.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      mod.author.toLowerCase().includes(searchQuery.value.toLowerCase())
    
    const matchesCat =
      selectedCategory.value === 'all' || mod.category === selectedCategory.value

    return matchesSearch && matchesCat
  })
})

const syncStats = computed(() => {
  const total = mods.value.length
  const synced = mods.value.filter((m) => m.status === 'synced').length
  const outdated = mods.value.filter((m) => m.status === 'outdated').length
  const missing = mods.value.filter((m) => m.status === 'missing').length
  return { total, synced, outdated, missing }
})

async function handleSyncAll() {
  await serversStore.syncServerMods(serverId.value)
}

function copyHash(hash: string) {
  navigator.clipboard.writeText(hash)
  toastStore.info('Empreinte copiée', `Hash : ${hash}`)
}

function copyInviteCode(code: string) {
  navigator.clipboard.writeText(code)
  toastStore.success('Code copié', `Code d'invitation "${code}" copié dans le presse-papier.`)
}
</script>

<template>
  <div v-if="server" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">
    
    <!-- Top Back Navigation -->
    <div class="flex items-center justify-between">
      <router-link
        to="/dashboard"
        class="inline-flex items-center gap-2 text-xs font-mono text-muted hover:text-foreground transition-colors"
      >
        <ArrowLeft class="w-4 h-4" />
        Retour aux serveurs
      </router-link>

      <router-link
        v-if="server.isOwner"
        :to="`/servers/${server.id}/admin`"
      >
        <Button size="sm" variant="outline" class="border-purple-800 text-purple-300 hover:bg-purple-950/40">
          <Shield class="w-3.5 h-3.5" />
          Gérer le Manifeste (Propriétaire)
        </Button>
      </router-link>
    </div>

    <!-- Server Header Banner -->
    <div class="industrial-panel p-6 bg-surface space-y-6">
      <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-4 border-b border-surface-border">
        <div>
          <div class="flex items-center gap-3">
            <h1 class="text-xl font-black uppercase tracking-wider text-foreground">
              {{ server.name }}
            </h1>
            <Badge v-if="server.status === 'synced'" variant="synced">
              🟢 Synchronisé
            </Badge>
            <Badge v-else-if="server.status === 'update_required'" variant="outdated">
              🟠 Mise à jour requise
            </Badge>
            <Badge v-else variant="error">
              🔴 Désynchronisé
            </Badge>
          </div>
          <div class="flex flex-wrap items-center gap-4 text-xs font-mono text-muted mt-2">
            <span>Carte : <strong class="text-foreground">{{ server.mapName }}</strong></span>
            <span>&bull;</span>
            <span>Version Requise : <strong class="text-primary">{{ server.modpackVersion }}</strong></span>
            <span>&bull;</span>
            <span>{{ server.modCount }} mods ({{ formatBytes(server.totalSizeBytes) }})</span>
            <template v-if="server.inviteCode">
              <span>&bull;</span>
              <span
                class="inline-flex items-center gap-1.5 px-2 py-0.5 bg-surface-raised border border-surface-border text-muted hover:text-foreground cursor-pointer"
                @click="copyInviteCode(server.inviteCode)"
                title="Cliquer pour copier le code d'invitation"
              >
                <span class="text-primary font-bold">CODE :</span>
                <span class="text-foreground font-bold">{{ server.inviteCode }}</span>
                <Copy class="w-3 h-3 text-muted hover:text-primary transition-colors" />
              </span>
            </template>
          </div>
        </div>

        <!-- Global Action CTA: "Synchroniser tout via l'application compagnon" -->
        <div class="flex items-center gap-3">
          <Button
            size="lg"
            variant="primary"
            @click="handleSyncAll"
            :disabled="companionStore.state.syncInProgress"
            class="shadow-lg shadow-primary/10"
          >
            <RefreshCw :class="['w-4 h-4', companionStore.state.syncInProgress ? 'animate-spin' : '']" />
            Synchroniser tout via l'app compagnon
          </Button>
        </div>
      </div>

      <!-- Sync Status Breakdown -->
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs font-mono">
        <div class="p-3 bg-surface-raised border border-surface-border flex items-center justify-between">
          <span class="text-muted">Total Requis :</span>
          <span class="font-bold text-foreground">{{ syncStats.total }} mods</span>
        </div>
        <div class="p-3 bg-surface-raised border border-surface-border flex items-center justify-between">
          <span class="text-emerald-400">À jour localement :</span>
          <span class="font-bold text-emerald-400">{{ syncStats.synced }}</span>
        </div>
        <div class="p-3 bg-surface-raised border border-surface-border flex items-center justify-between">
          <span class="text-amber-400">Version obsolète :</span>
          <span class="font-bold text-amber-400">{{ syncStats.outdated }}</span>
        </div>
        <div class="p-3 bg-surface-raised border border-surface-border flex items-center justify-between">
          <span class="text-rose-400">Mods manquants :</span>
          <span class="font-bold text-rose-400">{{ syncStats.missing }}</span>
        </div>
      </div>
    </div>

    <!-- Modpack Filter & Search Controls -->
    <div class="industrial-panel p-4 bg-surface flex flex-col sm:flex-row items-center justify-between gap-3">
      <!-- Search input -->
      <div class="relative w-full sm:w-80">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Rechercher un mod, fichier .zip, auteur..."
          class="w-full bg-surface-raised border border-surface-border rounded-sm px-3 py-2 pl-9 text-xs text-foreground focus:outline-none focus:border-primary font-mono"
        />
        <Search class="w-4 h-4 absolute left-2.5 top-2.5 text-muted pointer-events-none" />
      </div>

      <!-- Category Filter -->
      <div class="flex items-center gap-1.5 overflow-x-auto w-full sm:w-auto pb-1 sm:pb-0">
        <button
          v-for="cat in ['all', 'Tractors', 'Harvesters', 'Tools', 'Gameplay']"
          :key="cat"
          type="button"
          @click="selectedCategory = cat"
          :class="[
            'px-2.5 py-1 text-xs rounded-sm uppercase tracking-wider font-mono transition-colors whitespace-nowrap',
            selectedCategory === cat
              ? 'bg-primary text-white font-bold'
              : 'bg-surface-raised text-muted hover:text-foreground border border-surface-border',
          ]"
        >
          {{ cat === 'all' ? 'Tous' : cat }}
        </button>
      </div>
    </div>

    <!-- Mod List Table (Cahier des charges: Vue Joueur La liste des mods) -->
    <div class="industrial-panel bg-surface overflow-hidden border border-surface-border">
      <div class="overflow-x-auto">
        <table class="w-full text-left text-xs">
          <thead class="bg-surface-raised border-b border-surface-border text-muted uppercase font-bold tracking-wider font-mono text-[11px]">
            <tr>
              <th class="py-3 px-4">Miniature</th>
              <th class="py-3 px-4">Mod & Fichier (.zip)</th>
              <th class="py-3 px-4">Catégorie / Auteur</th>
              <th class="py-3 px-4 text-center">Version Serveur</th>
              <th class="py-3 px-4 text-center">Version Locale</th>
              <th class="py-3 px-4 text-center">Statut Client</th>
              <th class="py-3 px-4 text-right">Téléchargement S3</th>
            </tr>
          </thead>

          <tbody class="divide-y divide-surface-border">
            <tr
              v-for="mod in filteredMods"
              :key="mod.id"
              :class="[
                'hover:bg-surface-raised/60 transition-colors',
                !mod.isEnabled ? 'opacity-40 bg-black/20' : '',
              ]"
            >
              <!-- Mod Miniature / Icon -->
              <td class="py-3 px-4">
                <div class="w-12 h-12 bg-surface-raised border border-surface-border rounded-sm overflow-hidden flex items-center justify-center shrink-0">
                  <img
                    v-if="mod.thumbnailUrl"
                    :src="mod.thumbnailUrl"
                    :alt="mod.name"
                    class="w-full h-full object-cover"
                    loading="lazy"
                  />
                  <FileArchive v-else class="w-6 h-6 text-muted" />
                </div>
              </td>

              <!-- Name & File name (.zip) -->
              <td class="py-3 px-4">
                <div class="font-bold text-foreground text-sm tracking-wide">
                  {{ mod.name }}
                </div>
                <div class="font-mono text-[11px] text-muted flex items-center gap-1.5 mt-0.5">
                  <FileArchive class="w-3.5 h-3.5 text-primary" />
                  <span>{{ mod.fileName }}</span>
                  <span class="text-muted/60">&bull;</span>
                  <span>{{ formatBytes(mod.sizeBytes) }}</span>
                </div>
              </td>

              <!-- Category & Author -->
              <td class="py-3 px-4">
                <span class="px-2 py-0.5 bg-surface-raised border border-surface-border rounded-sm font-mono text-[11px] text-foreground">
                  {{ mod.category }}
                </span>
                <div class="text-[11px] text-muted mt-1">
                  par {{ mod.author }}
                </div>
              </td>

              <!-- Version requise par le serveur -->
              <td class="py-3 px-4 text-center">
                <span class="font-mono font-bold text-primary px-2 py-1 bg-surface-raised border border-surface-border rounded-sm">
                  v{{ mod.requiredVersion }}
                </span>
              </td>

              <!-- Version locale détectée par l'app Rust -->
              <td class="py-3 px-4 text-center font-mono">
                <span v-if="mod.localVersion" class="text-foreground">
                  v{{ mod.localVersion }}
                </span>
                <span v-else class="text-muted italic">
                  Non installé
                </span>
              </td>

              <!-- Statut vérifié via l'app Rust -->
              <td class="py-3 px-4 text-center">
                <Badge v-if="mod.status === 'synced'" variant="synced">
                  <CheckCircle2 class="w-3 h-3 text-emerald-400" />
                  À jour
                </Badge>
                <Badge v-else-if="mod.status === 'outdated'" variant="outdated">
                  <AlertTriangle class="w-3 h-3 text-amber-400" />
                  Version différente
                </Badge>
                <Badge v-else variant="missing">
                  <XCircle class="w-3 h-3 text-rose-400" />
                  Manquant
                </Badge>
              </td>

              <!-- Lien de téléchargement direct S3 -->
              <td class="py-3 px-4 text-right">
                <div class="flex items-center justify-end gap-2">
                  <button
                    type="button"
                    @click="copyHash(mod.hashSha256)"
                    class="p-1.5 text-muted hover:text-foreground bg-surface-raised border border-surface-border rounded-sm"
                    title="Copier le SHA-256"
                  >
                    <Copy class="w-3.5 h-3.5" />
                  </button>

                  <a
                    :href="mod.downloadUrl"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="inline-flex items-center gap-1 px-2.5 py-1.5 bg-surface-raised hover:bg-surface-hover border border-surface-border text-foreground hover:text-primary rounded-sm transition-colors text-xs font-mono"
                    title="Lien direct S3"
                  >
                    <DownloadCloud class="w-3.5 h-3.5" />
                    <span>S3 Direct</span>
                  </a>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Empty State -->
      <div v-if="filteredMods.length === 0" class="p-8 text-center text-muted font-mono text-xs">
        Aucun mod ne correspond à votre filtre de recherche.
      </div>
    </div>
  </div>

  <div v-else class="max-w-7xl mx-auto px-4 py-16 text-center">
    <h2 class="text-lg font-bold text-foreground">Serveur introuvable</h2>
    <router-link to="/dashboard" class="text-primary text-xs font-mono mt-2 inline-block">
      &larr; Revenir à la liste des serveurs
    </router-link>
  </div>
</template>
