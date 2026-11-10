<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useServersStore } from '@/stores/servers'
import { useToastStore } from '@/stores/toast'
import { formatBytes } from '@/lib/utils'
import {
  ShieldAlert,
  ShieldCheck,
  ArrowLeft,
  ArrowUp,
  ArrowDown,
  Trash2,
  Power,
  Layers,
  Sparkles,
  FileCode,
} from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
import Dropzone from '@/components/ui/Dropzone.vue'

const route = useRoute()
const serversStore = useServersStore()
const toastStore = useToastStore()

const serverId = computed(() => route.params.id as string)
const server = serversStore.getServerById(serverId.value)
const mods = serversStore.getModsForServer(serverId.value)

onMounted(async () => {
  await serversStore.fetchServers()
  await serversStore.fetchManifest(serverId.value)
})

const newVersionInput = ref('')

function handleFileUpload(file: File | { name: string; size: number }) {
  if (!server.value?.isOwner) {
    toastStore.error('Action refusée', 'Seul le propriétaire de ce serveur peut ajouter des mods.')
    return
  }
  serversStore.uploadModZip(serverId.value, file)
}

function handleToggleEnable(modId: string) {
  if (!server.value?.isOwner) {
    toastStore.error('Action refusée', 'Seul le propriétaire de ce serveur peut modifier les mods.')
    return
  }
  serversStore.toggleModEnabled(serverId.value, modId)
}

function handleDeleteMod(modId: string) {
  if (!server.value?.isOwner) {
    toastStore.error('Action refusée', 'Seul le propriétaire de ce serveur peut supprimer des mods.')
    return
  }
  if (confirm('Êtes-vous sûr de vouloir retirer ce mod du manifeste du serveur ?')) {
    serversStore.deleteMod(serverId.value, modId)
  }
}

function handleReorder(modId: string, direction: 'up' | 'down') {
  if (!server.value?.isOwner) return
  serversStore.moveModOrder(serverId.value, modId, direction)
}

function handleBumpVersion() {
  if (!server.value?.isOwner) {
    toastStore.error('Action refusée', 'Seul le propriétaire de ce serveur peut publier une nouvelle version.')
    return
  }
  if (!newVersionInput.value.trim()) {
    toastStore.warning('Format invalide', 'Veuillez saisir une version valide (ex: v2.5.0).')
    return
  }
  serversStore.bumpModpackVersion(serverId.value, newVersionInput.value.trim())
  newVersionInput.value = ''
}

function autoIncrementVersion() {
  if (!server.value) return
  const current = server.value.modpackVersion
  const parts = current.replace('v', '').split('.').map(Number)
  if (parts.length === 3) {
    parts[1] += 1
    parts[2] = 0
    newVersionInput.value = `v${parts.join('.')}`
  } else {
    newVersionInput.value = `${current}.1`
  }
}
</script>

<template>
  <div v-if="server" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
    
    <!-- Top Back Navigation & Admin Status -->
    <div class="flex items-center justify-between pb-2">
      <router-link
        :to="`/servers/${server.id}`"
        class="inline-flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-muted hover:text-foreground transition-colors font-display"
      >
        <span class="w-7 h-7 rounded-full bg-surface border border-surface-border flex items-center justify-center text-foreground hover:border-primary transition-colors">
          <ArrowLeft class="w-3.5 h-3.5" />
        </span>
        <span>Retour au serveur</span>
      </router-link>

      <div class="flex items-center gap-2">
        <Badge :variant="server.isOwner ? 'admin' : 'missing'">
          <ShieldCheck class="w-3.5 h-3.5" />
          {{ server.isOwner ? 'Propriétaire' : 'Accès Non Autorisé' }}
        </Badge>
      </div>
    </div>

    <!-- Access Denied Card if not server owner -->
    <div
      v-if="!server.isOwner"
      class="bg-surface rounded-2xl p-8 border border-destructive/40 text-center flex flex-col items-center justify-center gap-3 shadow-swiss-sm"
    >
      <div class="w-12 h-12 rounded-full bg-destructive/10 border border-destructive/30 flex items-center justify-center text-destructive">
        <ShieldAlert class="w-6 h-6" />
      </div>
      <h2 class="text-base font-bold uppercase tracking-wider text-foreground font-display">
        Accès Réservé au Propriétaire
      </h2>
      <p class="text-xs text-muted max-w-md font-sans">
        Vous n'êtes pas le propriétaire du serveur <strong>{{ server.name }}</strong>.
      </p>
      <router-link :to="`/servers/${server.id}`" class="mt-2">
        <Button size="sm" variant="secondary" class="rounded-full">
          <ArrowLeft class="w-4 h-4" />
          Retour au serveur
        </Button>
      </router-link>
    </div>

    <template v-else>
      <!-- Header Section -->
      <div class="bg-surface rounded-2xl p-6 sm:p-8 border border-surface-border flex flex-col md:flex-row md:items-center justify-between gap-6 shadow-swiss-sm">
        <div>
          <h1 class="text-2xl sm:text-3xl font-black uppercase tracking-tight text-foreground font-display">
            Administration : {{ server.name }}
          </h1>
          <p class="text-xs text-muted font-sans mt-1">
            Gérez les mods, l'ordre de chargement et les versions du serveur.
          </p>
        </div>

        <!-- Version Bump Box -->
        <div class="p-4 rounded-xl bg-surface-raised border border-surface-border flex flex-col gap-2 shrink-0">
          <div class="text-[10px] text-muted uppercase font-bold tracking-wider font-mono">
            Version Publiée : <span class="text-primary font-bold">v{{ server.modpackVersion }}</span>
          </div>
          <div class="flex items-center gap-1.5">
            <input
              v-model="newVersionInput"
              type="text"
              placeholder="ex: v2.5.0"
              class="w-28 bg-surface border border-surface-border rounded-lg px-2.5 py-1.5 text-xs text-foreground font-mono focus:outline-none focus:border-primary"
            />
            <Button size="sm" variant="outline" class="rounded-lg" @click="autoIncrementVersion" title="Incrémenter +1 minor">
              <Sparkles class="w-3.5 h-3.5 text-primary" />
            </Button>
            <Button size="sm" variant="primary" class="rounded-lg" @click="handleBumpVersion">
              Publier
            </Button>
          </div>
        </div>
      </div>

      <!-- Section 1: Drag & Drop Zone -->
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <h2 class="text-sm font-bold uppercase tracking-wider text-foreground flex items-center gap-2 font-display">
            <FileCode class="w-4 h-4 text-primary" />
            Téléversement de Mods (.zip)
          </h2>
          <span class="text-xs text-muted font-mono">
            Extraction XML & stockage objet S3
          </span>
        </div>

        <Dropzone @file-selected="handleFileUpload" />
      </div>

      <!-- Section 2: Manifest Management Table -->
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <h2 class="text-sm font-bold uppercase tracking-wider text-foreground flex items-center gap-2 font-display">
            <Layers class="w-4 h-4 text-primary" />
            Manifeste Actif ({{ mods.length }} mods enregistrés)
          </h2>
          <span class="text-xs text-muted font-mono">
            Ordre d'exécution & statut de distribution
          </span>
        </div>

        <div class="bg-surface rounded-2xl border border-surface-border overflow-hidden shadow-swiss-sm">
          <div class="overflow-x-auto">
            <table class="w-full text-left text-xs">
              <thead class="bg-surface-raised border-b border-surface-border text-muted uppercase font-bold tracking-wider font-display text-[10px]">
                <tr>
                  <th class="py-3 px-4 text-center w-16">Ordre</th>
                  <th class="py-3 px-4">Mod & Fichier ZIP</th>
                  <th class="py-3 px-3">Catégorie</th>
                  <th class="py-3 px-3">Version modDesc</th>
                  <th class="py-3 px-3">Taille</th>
                  <th class="py-3 px-3 text-center">Statut dans le pack</th>
                  <th class="py-3 px-4 text-right">Actions</th>
                </tr>
              </thead>

              <tbody class="divide-y divide-surface-border">
                <tr
                  v-for="(mod, index) in mods"
                  :key="mod.id"
                  :class="[
                    'hover:bg-surface-raised/40 transition-colors',
                    !mod.isEnabled ? 'bg-red-500/5 opacity-60' : '',
                  ]"
                >
                  <!-- Order & Reorder Controls -->
                  <td class="py-3 px-4 text-center">
                    <div class="flex items-center justify-center gap-1.5">
                      <span class="font-bold text-muted font-mono w-4">{{ index + 1 }}</span>
                      <div class="flex flex-col">
                        <button
                          type="button"
                          :disabled="index === 0"
                          @click="handleReorder(mod.id, 'up')"
                          class="p-0.5 text-muted hover:text-foreground disabled:opacity-20 transition-colors"
                          title="Monter"
                        >
                          <ArrowUp class="w-3 h-3" />
                        </button>
                        <button
                          type="button"
                          :disabled="index === mods.length - 1"
                          @click="handleReorder(mod.id, 'down')"
                          class="p-0.5 text-muted hover:text-foreground disabled:opacity-20 transition-colors"
                          title="Descendre"
                        >
                          <ArrowDown class="w-3 h-3" />
                        </button>
                      </div>
                    </div>
                  </td>

                  <!-- Mod Name & File -->
                  <td class="py-3 px-4">
                    <div class="font-bold text-foreground text-sm font-display">
                      {{ mod.name }}
                    </div>
                    <div class="text-[11px] text-muted flex items-center gap-2 mt-0.5 font-mono">
                      <span class="text-primary font-bold">{{ mod.fileName }}</span>
                      <span class="text-muted/40">•</span>
                      <span class="truncate max-w-[200px]" :title="mod.hashSha256">
                        SHA: {{ mod.hashSha256.substring(0, 12) }}...
                      </span>
                    </div>
                  </td>

                  <!-- Category -->
                  <td class="py-3 px-3">
                    <span class="px-2 py-0.5 bg-surface-raised border border-surface-border text-foreground rounded-full text-[10px] font-mono font-semibold">
                      {{ mod.category }}
                    </span>
                  </td>

                  <!-- Version -->
                  <td class="py-3 px-3 font-bold text-primary font-mono">
                    v{{ mod.requiredVersion }}
                  </td>

                  <!-- Size -->
                  <td class="py-3 px-3 text-muted font-mono">
                    {{ formatBytes(mod.sizeBytes) }}
                  </td>

                  <!-- Toggle Active -->
                  <td class="py-3 px-3 text-center">
                    <button
                      type="button"
                      @click="handleToggleEnable(mod.id)"
                      :class="[
                        'px-3 py-1 rounded-full border text-[11px] font-bold uppercase tracking-wider transition-colors inline-flex items-center gap-1.5',
                        mod.isEnabled
                          ? 'bg-primary/10 border-primary/30 text-primary hover:bg-primary/20'
                          : 'bg-surface-raised border-surface-border text-muted hover:text-foreground',
                      ]"
                    >
                      <Power class="w-3 h-3" />
                      {{ mod.isEnabled ? 'Actif' : 'Désactivé' }}
                    </button>
                  </td>

                  <!-- Delete -->
                  <td class="py-3 px-4 text-right">
                    <Button
                      size="sm"
                      variant="ghost"
                      @click="handleDeleteMod(mod.id)"
                      class="text-destructive hover:bg-destructive/10 rounded-full"
                      title="Retirer du modpack"
                    >
                      <Trash2 class="w-3.5 h-3.5" />
                      Retirer
                    </Button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
