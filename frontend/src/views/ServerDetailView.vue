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
  toastStore.info('Empreinte SHA-256 copiée', `Hash : ${hash}`)
}

function copyInviteCode(code: string) {
  navigator.clipboard.writeText(code)
  toastStore.success('Code copié', `Code d'invitation "${code}" copié dans le presse-papier.`)
}
</script>

<template>
  <div v-if="server" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">
    
    <!-- Top Back Navigation & Admin Button -->
    <div class="flex items-center justify-between pb-2">
      <router-link
        to="/dashboard"
        class="inline-flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-muted hover:text-foreground transition-colors font-display"
      >
        <span class="w-7 h-7 rounded-full bg-surface border border-surface-border flex items-center justify-center text-foreground hover:border-primary transition-colors">
          <ArrowLeft class="w-3.5 h-3.5" />
        </span>
        <span>Retour aux serveurs</span>
      </router-link>

      <router-link
        v-if="server.isOwner"
        :to="`/servers/${server.id}/admin`"
      >
        <Button size="sm" variant="dark" class="rounded-full">
          <Shield class="w-3.5 h-3.5 text-primary" />
          Gérer le serveur
        </Button>
      </router-link>
    </div>

    <!-- Server Header Banner -->
    <div class="bg-surface border border-surface-border rounded-2xl p-6 sm:p-8 space-y-6 shadow-swiss-sm">
      <div class="flex flex-col md:flex-row md:items-start justify-between gap-6 pb-6 border-b border-surface-border">
        <div>
          <div class="flex items-center gap-3">
            <h1 class="text-2xl sm:text-3xl font-black uppercase tracking-tight text-foreground font-display">
              {{ server.name }}
            </h1>
            <Badge v-if="server.status === 'synced'" variant="synced">
              ● Synchronisé
            </Badge>
            <Badge v-else-if="server.status === 'update_required'" variant="outdated">
              ○ Mise à jour
            </Badge>
            <Badge v-else variant="error">
              ✕ Désynchronisé
            </Badge>
          </div>

          <div class="flex flex-wrap items-center gap-3 text-xs font-mono text-muted mt-2">
            <span class="text-primary font-bold">v{{ server.modpackVersion }}</span>
            <span>•</span>
            <span>Carte : {{ server.mapName }}</span>
            <span>•</span>
            <span>{{ server.address }}</span>
            <span>•</span>
            <span>{{ server.modCount }} mods</span>
            <span>•</span>
            <span>{{ formatBytes(server.totalSizeBytes) }}</span>

            <template v-if="server.inviteCode">
              <span>•</span>
              <button
                type="button"
                class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-surface-raised border border-surface-border text-muted hover:text-foreground text-[11px] cursor-pointer"
                @click="copyInviteCode(server.inviteCode)"
                title="Cliquer pour copier le code d'invitation"
              >
                <span class="text-primary font-bold">CODE:</span>
                <span class="text-foreground font-bold">{{ server.inviteCode }}</span>
                <Copy class="w-3 h-3 text-muted hover:text-primary transition-colors" />
              </button>
            </template>
          </div>
        </div>

        <!-- Global Action CTA: Synchroniser tout -->
        <div class="flex items-center gap-3 shrink-0">
          <Button
            size="lg"
            variant="primary"
            @click="handleSyncAll"
            :disabled="companionStore.state.syncInProgress"
            class="rounded-full shadow-swiss-md"
          >
            <RefreshCw :class="['w-4 h-4', companionStore.state.syncInProgress ? 'animate-spin' : '']" />
            Synchroniser tout le modpack
          </Button>
        </div>
      </div>

      <!-- Sync Status Breakdown (Clean Swiss Stat Boxes) -->
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
        <div class="p-4 rounded-xl bg-surface-raised border border-surface-border">
          <div class="text-[10px] font-bold uppercase tracking-wider text-muted font-display">Total Requis</div>
          <div class="text-xl font-black font-display text-foreground mt-1">
            {{ syncStats.total }} <span class="text-xs font-normal text-muted">mods</span>
          </div>
        </div>

        <div class="p-4 rounded-xl bg-surface-raised border border-surface-border">
          <div class="text-[10px] font-bold uppercase tracking-wider text-emerald-600 dark:text-emerald-400 font-display">À jour localement</div>
          <div class="text-xl font-black font-display text-emerald-600 dark:text-emerald-400 mt-1">
            {{ syncStats.synced }} <span class="text-xs font-normal text-muted">mods</span>
          </div>
        </div>

        <div class="p-4 rounded-xl bg-surface-raised border border-surface-border">
          <div class="text-[10px] font-bold uppercase tracking-wider text-amber-600 dark:text-amber-400 font-display">Version obsolète</div>
          <div class="text-xl font-black font-display text-amber-600 dark:text-amber-400 mt-1">
            {{ syncStats.outdated }} <span class="text-xs font-normal text-muted">mods</span>
          </div>
        </div>

        <div class="p-4 rounded-xl bg-surface-raised border border-surface-border">
          <div class="text-[10px] font-bold uppercase tracking-wider text-rose-600 dark:text-rose-400 font-display">Mods manquants</div>
          <div class="text-xl font-black font-display text-rose-600 dark:text-rose-400 mt-1">
            {{ syncStats.missing }} <span class="text-xs font-normal text-muted">mods</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Modpack Filter & Search Controls -->
    <div class="bg-surface border border-surface-border rounded-xl p-3 sm:p-4 flex flex-col sm:flex-row items-center justify-between gap-3 shadow-swiss-sm">
      <!-- Search input -->
      <div class="relative w-full sm:w-80">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Rechercher un mod, fichier .zip, auteur..."
          class="w-full bg-surface-raised border border-surface-border rounded-lg px-3 py-2 pl-9 text-xs text-foreground focus:outline-none focus:border-primary font-sans placeholder:text-muted"
        />
        <Search class="w-4 h-4 absolute left-3 top-2.5 text-muted pointer-events-none" />
      </div>

      <!-- Category Filter Pills -->
      <div class="flex items-center gap-1.5 overflow-x-auto w-full sm:w-auto pb-1 sm:pb-0">
        <button
          v-for="cat in ['all', 'Tractors', 'Harvesters', 'Tools', 'Gameplay']"
          :key="cat"
          type="button"
          @click="selectedCategory = cat"
          :class="[
            'px-3 py-1.5 text-xs rounded-full uppercase tracking-wider font-bold transition-all whitespace-nowrap',
            selectedCategory === cat
              ? 'bg-brand-dark text-white dark:bg-white dark:text-brand-dark shadow-sm'
              : 'bg-surface-raised text-muted hover:text-foreground border border-surface-border',
          ]"
        >
          {{ cat === 'all' ? 'Tous les mods' : cat }}
        </button>
      </div>
    </div>

    <!-- Mod List Table (Inspired by Reference 1: E-KOOL Bottom Tabular Layout) -->
    <div class="bg-surface rounded-2xl overflow-hidden border border-surface-border shadow-swiss-sm">
      <div class="overflow-x-auto">
        <table class="w-full text-left text-xs">
          <thead class="bg-surface-raised border-b border-surface-border text-muted uppercase font-bold tracking-wider font-display text-[10px]">
            <tr>
              <th class="py-3.5 px-4 w-16">Miniature</th>
              <th class="py-3.5 px-4">Mod & Archive ZIP</th>
              <th class="py-3.5 px-4">Catégorie / Auteur</th>
              <th class="py-3.5 px-4 text-center">Version Serveur</th>
              <th class="py-3.5 px-4 text-center">Version Locale</th>
              <th class="py-3.5 px-4 text-center">Statut Sync</th>
              <th class="py-3.5 px-4 text-right">Téléchargement S3</th>
            </tr>
          </thead>

          <tbody class="divide-y divide-surface-border">
            <tr
              v-for="mod in filteredMods"
              :key="mod.id"
              :class="[
                'hover:bg-surface-raised/60 transition-colors',
                !mod.isEnabled ? 'opacity-40 bg-black/5 dark:bg-black/20' : '',
              ]"
            >
              <!-- Thumbnail -->
              <td class="py-3.5 px-4">
                <div class="w-11 h-11 bg-surface-raised border border-surface-border rounded-lg overflow-hidden flex items-center justify-center shrink-0">
                  <img
                    v-if="mod.thumbnailUrl"
                    :src="mod.thumbnailUrl"
                    :alt="mod.name"
                    class="w-full h-full object-cover"
                    loading="lazy"
                  />
                  <FileArchive v-else class="w-5 h-5 text-muted" />
                </div>
              </td>

              <!-- Name & File name (.zip) -->
              <td class="py-3.5 px-4">
                <div class="font-bold text-foreground text-sm tracking-tight font-display">
                  {{ mod.name }}
                </div>
                <div class="font-mono text-[11px] text-muted flex items-center gap-1.5 mt-0.5">
                  <FileArchive class="w-3.5 h-3.5 text-primary" />
                  <span>{{ mod.fileName }}</span>
                  <span class="text-muted/60">•</span>
                  <span>{{ formatBytes(mod.sizeBytes) }}</span>
                </div>
              </td>

              <!-- Category & Author -->
              <td class="py-3.5 px-4">
                <span class="px-2 py-0.5 bg-surface-raised border border-surface-border rounded-full font-mono text-[10px] text-foreground font-semibold">
                  {{ mod.category }}
                </span>
                <div class="text-[11px] text-muted mt-1">
                  par {{ mod.author }}
                </div>
              </td>

              <!-- Required Server Version -->
              <td class="py-3.5 px-4 text-center">
                <span class="font-mono font-bold text-primary px-2.5 py-1 bg-primary/10 border border-primary/20 rounded-full text-xs">
                  v{{ mod.requiredVersion }}
                </span>
              </td>

              <!-- Local Version -->
              <td class="py-3.5 px-4 text-center font-mono">
                <span v-if="mod.localVersion" class="text-foreground font-semibold">
                  v{{ mod.localVersion }}
                </span>
                <span v-else class="text-muted italic">
                  Non installé
                </span>
              </td>

              <!-- Sync Status -->
              <td class="py-3.5 px-4 text-center">
                <Badge v-if="mod.status === 'synced'" variant="synced">
                  ● À jour
                </Badge>
                <Badge v-else-if="mod.status === 'outdated'" variant="outdated">
                  ○ Obsolète
                </Badge>
                <Badge v-else variant="missing">
                  ✕ Manquant
                </Badge>
              </td>

              <!-- S3 Direct Download & Hash -->
              <td class="py-3.5 px-4 text-right">
                <div class="flex items-center justify-end gap-2">
                  <button
                    type="button"
                    @click="copyHash(mod.hashSha256)"
                    class="p-1.5 text-muted hover:text-foreground bg-surface-raised border border-surface-border rounded-lg transition-colors"
                    title="Copier l'empreinte SHA-256"
                  >
                    <Copy class="w-3.5 h-3.5" />
                  </button>

                  <a
                    :href="mod.downloadUrl"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-surface-raised hover:bg-surface-hover border border-surface-border text-foreground hover:text-primary rounded-lg transition-colors text-xs font-mono font-bold"
                    title="Télécharger depuis S3"
                  >
                    <DownloadCloud class="w-3.5 h-3.5 text-primary" />
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
    <h2 class="text-lg font-bold text-foreground font-display uppercase">Serveur introuvable</h2>
    <router-link to="/dashboard" class="text-primary text-xs font-mono mt-2 inline-block">
      ← Revenir au tableau de bord
    </router-link>
  </div>
</template>
