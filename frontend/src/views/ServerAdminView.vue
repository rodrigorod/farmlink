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
    parts[1] += 1 // Bump minor version
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
    <div class="flex items-center justify-between">
      <router-link
        :to="`/servers/${server.id}`"
        class="inline-flex items-center gap-2 text-xs font-mono text-muted hover:text-foreground transition-colors"
      >
        <ArrowLeft class="w-4 h-4" />
        Retour à la vue Joueur
      </router-link>

      <div class="flex items-center gap-2">
        <Badge :variant="server.isOwner ? 'admin' : 'missing'">
          <ShieldCheck class="w-3.5 h-3.5" />
          {{ server.isOwner ? 'Propriétaire du Serveur' : 'Accès Non Autorisé' }}
        </Badge>
      </div>
    </div>

    <!-- Access Denied Card if not server owner -->
    <div
      v-if="!server.isOwner"
      class="industrial-panel p-8 bg-surface border border-destructive/50 text-center flex flex-col items-center justify-center gap-3"
    >
      <div class="w-12 h-12 rounded-full bg-destructive/10 border border-destructive/40 flex items-center justify-center text-destructive">
        <ShieldAlert class="w-6 h-6" />
      </div>
      <h2 class="text-base font-bold uppercase tracking-wider text-foreground">
        Accès Réservé au Propriétaire
      </h2>
      <p class="text-xs text-muted max-w-md">
        Vous n'êtes pas le propriétaire du serveur <strong>{{ server.name }}</strong>. Seul le propriétaire peut téléverser de nouveaux mods, modifier les statuts d'activation ou publier des versions de manifeste.
      </p>
      <router-link :to="`/servers/${server.id}`" class="mt-2">
        <Button size="sm" variant="secondary">
          <ArrowLeft class="w-4 h-4" />
          Retour à la vue joueur
        </Button>
      </router-link>
    </div>

    <template v-else>
      <!-- Header Section -->
    <div class="industrial-panel p-6 bg-surface flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 bg-purple-950/60 border border-purple-800 rounded-sm flex items-center justify-center text-purple-300">
            <Layers class="w-5 h-5" />
          </div>
          <div>
            <h1 class="text-xl font-black uppercase tracking-wider text-foreground">
              Panneau d'Administration du Modpack
            </h1>
            <p class="text-xs text-muted font-mono mt-0.5">
              Serveur : <span class="text-foreground font-bold">{{ server.name }}</span> &bull; Manifeste FS25
            </p>
          </div>
        </div>
      </div>

      <!-- Quick Version Bump Box -->
      <div class="industrial-panel p-3 bg-surface-raised border border-surface-border flex items-center gap-2">
        <div>
          <div class="text-[10px] text-muted uppercase font-bold tracking-wider font-mono">
            Version Actuelle : <span class="text-primary">{{ server.modpackVersion }}</span>
          </div>
          <div class="flex items-center gap-1.5 mt-1">
            <input
              v-model="newVersionInput"
              type="text"
              placeholder="ex: v2.5.0"
              class="w-24 bg-surface border border-surface-border rounded-sm px-2 py-1 text-xs text-foreground font-mono focus:outline-none focus:border-primary"
            />
            <Button size="sm" variant="outline" @click="autoIncrementVersion" title="Générer +1 minor">
              <Sparkles class="w-3.5 h-3.5 text-primary" />
            </Button>
            <Button size="sm" variant="primary" @click="handleBumpVersion">
              Publier Version
            </Button>
          </div>
        </div>
      </div>
    </div>

    <!-- Section 1: Drag & Drop Zone (Upload .zip + Backend Go XML Parsing) -->
    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-bold uppercase tracking-wider text-foreground flex items-center gap-2">
          <FileCode class="w-4 h-4 text-primary" />
          Téléversement & Indexation de Nouveaux Mods
        </h2>
        <span class="text-xs text-muted font-mono">
          Analyse automatique du descVersion & stockage S3
        </span>
      </div>

      <Dropzone @file-selected="handleFileUpload" />
    </div>

    <!-- Section 2: Manifest Management Table (Reorder, Toggle Active, Delete) -->
    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-bold uppercase tracking-wider text-foreground flex items-center gap-2">
          <Layers class="w-4 h-4 text-primary" />
          Manifeste du Serveur ({{ mods.length }} mods enregistrés)
        </h2>
        <span class="text-xs text-muted font-mono">
          Ordre de chargement des dépendances & statut de publication
        </span>
      </div>

      <div class="industrial-panel bg-surface border border-surface-border overflow-hidden">
        <div class="overflow-x-auto">
          <table class="w-full text-left text-xs font-mono">
            <thead class="bg-surface-raised border-b border-surface-border text-muted uppercase font-bold tracking-wider text-[11px]">
              <tr>
                <th class="py-3 px-3 text-center w-16">Ordre</th>
                <th class="py-3 px-4">Nom du Mod & Archive</th>
                <th class="py-3 px-3">Catégorie</th>
                <th class="py-3 px-3">Version modDesc</th>
                <th class="py-3 px-3">Taille</th>
                <th class="py-3 px-3 text-center">Actif dans le Pack</th>
                <th class="py-3 px-4 text-right">Actions</th>
              </tr>
            </thead>

            <tbody class="divide-y divide-surface-border">
              <tr
                v-for="(mod, index) in mods"
                :key="mod.id"
                :class="[
                  'hover:bg-surface-raised/40 transition-colors',
                  !mod.isEnabled ? 'bg-red-950/10 opacity-60' : '',
                ]"
              >
                <!-- Order & Reorder Controls -->
                <td class="py-3 px-3 text-center">
                  <div class="flex items-center justify-center gap-1">
                    <span class="font-bold text-muted w-4">{{ index + 1 }}</span>
                    <div class="flex flex-col">
                      <button
                        type="button"
                        :disabled="index === 0"
                        @click="handleReorder(mod.id, 'up')"
                        class="p-0.5 text-muted hover:text-foreground disabled:opacity-20"
                        title="Monter"
                      >
                        <ArrowUp class="w-3 h-3" />
                      </button>
                      <button
                        type="button"
                        :disabled="index === mods.length - 1"
                        @click="handleReorder(mod.id, 'down')"
                        class="p-0.5 text-muted hover:text-foreground disabled:opacity-20"
                        title="Descendre"
                      >
                        <ArrowDown class="w-3 h-3" />
                      </button>
                    </div>
                  </div>
                </td>

                <!-- Mod Name & File -->
                <td class="py-3 px-4">
                  <div class="font-sans font-bold text-foreground text-sm">
                    {{ mod.name }}
                  </div>
                  <div class="text-[11px] text-muted flex items-center gap-2 mt-0.5">
                    <span class="text-primary">{{ mod.fileName }}</span>
                    <span class="text-muted/40">&bull;</span>
                    <span class="text-muted/80 truncate max-w-[200px]" :title="mod.hashSha256">
                      SHA: {{ mod.hashSha256.substring(0, 12) }}...
                    </span>
                  </div>
                </td>

                <!-- Category -->
                <td class="py-3 px-3">
                  <span class="px-2 py-0.5 bg-surface-raised border border-surface-border text-foreground rounded-sm text-[11px]">
                    {{ mod.category }}
                  </span>
                </td>

                <!-- Version extracted from modDesc.xml -->
                <td class="py-3 px-3 font-bold text-primary">
                  v{{ mod.requiredVersion }}
                </td>

                <!-- Size -->
                <td class="py-3 px-3 text-muted">
                  {{ formatBytes(mod.sizeBytes) }}
                </td>

                <!-- Toggle Active/Inactive (without deleting from DB) -->
                <td class="py-3 px-3 text-center">
                  <button
                    type="button"
                    @click="handleToggleEnable(mod.id)"
                    :class="[
                      'px-2.5 py-1 rounded-sm border text-[11px] font-bold uppercase transition-colors inline-flex items-center gap-1.5',
                      mod.isEnabled
                        ? 'bg-emerald-950/60 border-emerald-800 text-emerald-400 hover:bg-emerald-900/60'
                        : 'bg-surface-raised border-surface-border text-muted hover:text-foreground',
                    ]"
                  >
                    <Power class="w-3 h-3" />
                    {{ mod.isEnabled ? 'Actif' : 'Désactivé' }}
                  </button>
                </td>

                <!-- Delete from manifest -->
                <td class="py-3 px-4 text-right">
                  <Button
                    size="sm"
                    variant="ghost"
                    @click="handleDeleteMod(mod.id)"
                    class="text-destructive hover:bg-destructive/10 hover:text-destructive"
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
