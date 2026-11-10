<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useServersStore } from '@/stores/servers'
import { useCompanionStore } from '@/stores/companion'
import { useToastStore } from '@/stores/toast'
import { formatBytes } from '@/lib/utils'
import {
  Server as ServerIcon,
  Search,
  Copy,
  Shield,
  Layers,
  Play,
  RotateCw,
  Plus,
  ArrowRight,
} from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
import GeometricSkeleton from '@/components/ui/GeometricSkeleton.vue'
import ServerActionModal from '@/components/servers/ServerActionModal.vue'

const serversStore = useServersStore()
const companionStore = useCompanionStore()
const toastStore = useToastStore()

const isLoading = ref(true)
const showServerModal = ref(false)
const searchQuery = ref('')
const selectedTld = ref('all')
const selectedStatusFilter = ref<'all' | 'synced' | 'update_required' | 'error_desync'>('all')

const syncedCount = computed(() => serversStore.servers.filter(s => s.status === 'synced').length)
const updateRequiredCount = computed(() => serversStore.servers.filter(s => s.status === 'update_required').length)
const errorCount = computed(() => serversStore.servers.filter(s => s.status === 'error_desync').length)

const totalModsCount = computed(() => {
  return serversStore.servers.reduce((acc, s) => acc + (s.modCount || 0), 0)
})

const totalModsSize = computed(() => {
  return serversStore.servers.reduce((acc, s) => acc + (s.totalSizeBytes || 0), 0)
})

const filteredServers = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  return serversStore.servers.filter((s) => {
    const addr = s.address.toLowerCase()
    const matchesSearch = !q || s.name.toLowerCase().includes(q) || addr.includes(q)
    const matchesTld = selectedTld.value === 'all' || addr.includes(selectedTld.value.toLowerCase())
    const matchesStatus = selectedStatusFilter.value === 'all' || s.status === selectedStatusFilter.value
    return matchesSearch && matchesTld && matchesStatus
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

function copyInviteCode(code: string) {
  navigator.clipboard.writeText(code)
  toastStore.success('Code copié', `Code d'invitation "${code}" copié. Partagez-le avec vos équipiers !`)
}

async function joinServer(server: any) {
  if (companionStore.state.isConnected) {
    toastStore.info(
      'Lancement via Rust Compagnon',
      `Ouverture de Farming Simulator 25 et connexion directe à "${server.name}" (${server.address})...`
    )
    await companionStore.launchGame(`farmlink://join?server=${server.id}&addr=${encodeURIComponent(server.address)}`)
  } else {
    copyAddress(server.address)
    toastStore.warning(
      'Compagnon Rust hors-ligne',
      `Adresse ${server.address} copiée. Rendez-vous dans "Paramètres" pour configurer votre dossier de jeu et activer le lancement 1-clic.`
    )
  }
}
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
    
    <!-- Header Title Row -->
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-4 border-b border-surface-border">
      <div>
        <h1 class="text-3xl sm:text-4xl font-black uppercase tracking-tight text-foreground font-display">
          Farming Simulator 25
        </h1>
        <p class="text-xs text-muted font-sans mt-0.5">
          Vos serveurs et modpacks synchronisés en temps réel.
        </p>
      </div>

      <!-- Action Button: Nouveau Serveur -->
      <div class="flex items-center gap-3">
        <Button
          size="md"
          variant="primary"
          @click="showServerModal = true"
          class="shadow-swiss-sm"
        >
          <Plus class="w-4 h-4" />
          Nouveau Serveur
        </Button>
      </div>
    </div>

    <!-- 3-Block Swiss Hero Layout (Inspired by Reference 1: E-KOOL Dashboard) -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-5 items-stretch">
      
      <!-- Block 1: Brand Green Block -->
      <div class="lg:col-span-5 bg-primary text-white rounded-2xl p-6 sm:p-7 shadow-swiss-md relative flex flex-col justify-between overflow-hidden group">
        <!-- Background subtle emblem watermark -->
        <img
          src="/logo_navbar.png"
          alt=""
          class="absolute -right-4 -bottom-4 w-48 h-auto opacity-15 pointer-events-none transition-transform group-hover:scale-105"
        />

        <div>
          <span class="text-xs font-bold uppercase tracking-wider text-white/80 font-display">
            État de synchronisation
          </span>

          <!-- Main Typographic Stats -->
          <div class="mt-3">
            <div class="text-4xl sm:text-5xl font-black font-display tracking-tighter leading-none">
              {{ syncedCount }} / {{ serversStore.servers.length }}
            </div>
            <h2 class="text-sm sm:text-base font-bold uppercase tracking-wider mt-1 text-white/95 font-display">
              Serveurs Synchronisés
            </h2>
            <p class="text-xs text-white/80 font-sans mt-1 max-w-sm">
              Tous vos mods sont vérifiés et prêts pour le jeu multijoueur.
            </p>
          </div>
        </div>

        <!-- Filter Buttons -->
        <div class="mt-6 pt-4 border-t border-white/20 flex flex-wrap items-center gap-1.5 z-10">
          <button
            type="button"
            @click="selectedStatusFilter = 'all'"
            :class="[
              'px-2.5 py-1 rounded-full text-[11px] font-bold uppercase tracking-wider transition-all',
              selectedStatusFilter === 'all'
                ? 'bg-white text-primary shadow-sm'
                : 'bg-white/15 text-white hover:bg-white/25 border border-white/20'
            ]"
          >
            Tous
          </button>
          <button
            type="button"
            @click="selectedStatusFilter = 'synced'"
            :class="[
              'px-2.5 py-1 rounded-full text-[11px] font-bold uppercase tracking-wider transition-all',
              selectedStatusFilter === 'synced'
                ? 'bg-white text-primary shadow-sm'
                : 'bg-white/15 text-white hover:bg-white/25 border border-white/20'
            ]"
          >
            À jour ({{ syncedCount }})
          </button>
          <button
            type="button"
            @click="selectedStatusFilter = 'update_required'"
            :class="[
              'px-2.5 py-1 rounded-full text-[11px] font-bold uppercase tracking-wider transition-all',
              selectedStatusFilter === 'update_required'
                ? 'bg-white text-primary shadow-sm'
                : 'bg-white/15 text-white hover:bg-white/25 border border-white/20'
            ]"
          >
            Mise à jour ({{ updateRequiredCount }})
          </button>
          <button
            v-if="errorCount > 0"
            type="button"
            @click="selectedStatusFilter = 'error_desync'"
            :class="[
              'px-2.5 py-1 rounded-full text-[11px] font-bold uppercase tracking-wider transition-all',
              selectedStatusFilter === 'error_desync'
                ? 'bg-white text-primary shadow-sm'
                : 'bg-white/15 text-white hover:bg-white/25 border border-white/20'
            ]"
          >
            Erreurs ({{ errorCount }})
          </button>
        </div>
      </div>

      <!-- Block 2: Modpack Storage Card -->
      <div class="lg:col-span-4 bg-surface-raised text-foreground rounded-2xl p-6 sm:p-7 border border-surface-border relative flex flex-col justify-between">
        <div>
          <span class="text-xs font-bold uppercase tracking-wider text-muted font-display">
            Volume des modpacks
          </span>

          <!-- Main Stat -->
          <div class="mt-3">
            <div class="text-3xl sm:text-4xl font-black font-display tracking-tight text-foreground">
              {{ totalModsCount }} <span class="text-sm font-bold text-muted uppercase tracking-normal">Mods répertoriés</span>
            </div>
            <div class="text-sm font-mono text-primary font-bold mt-1">
              {{ formatBytes(totalModsSize) }} d'espace disque
            </div>
            <p class="text-xs text-muted mt-2 font-sans leading-relaxed">
              Fichiers .zip automatiquement vérifiés par somme de contrôle SHA-256.
            </p>
          </div>
        </div>

        <div class="mt-6 pt-4 border-t border-surface-border flex items-center justify-between text-xs text-muted font-mono">
          <span>{{ serversStore.servers.length }} serveurs enregistrés</span>
        </div>
      </div>

      <!-- Block 3: Local Companion Status Card -->
      <div class="lg:col-span-3 bg-surface text-foreground rounded-2xl p-6 sm:p-7 border border-surface-border relative flex flex-col justify-between shadow-swiss-sm">
        <div>
          <!-- Header -->
          <div class="flex items-center justify-between">
            <span class="text-xs font-bold uppercase tracking-wider text-muted font-display">
              Compagnon Local
            </span>
            <div
              :class="[
                'w-2.5 h-2.5 rounded-full',
                companionStore.state.isConnected ? 'bg-primary ring-4 ring-primary/20' : 'bg-rose-500'
              ]"
            ></div>
          </div>

          <!-- Big Stat -->
          <div class="mt-3">
            <div class="text-3xl sm:text-4xl font-black font-display tracking-tight text-foreground leading-none">
              {{ companionStore.state.isConnected ? 'Connecté' : 'Hors-ligne' }}
            </div>
            <div class="text-xs font-bold uppercase tracking-wider text-foreground font-display mt-2">
              {{ companionStore.state.isConnected ? 'Moteur de jeu actif' : 'Moteur inactif' }}
            </div>
            <p class="text-xs text-muted mt-1 font-sans">
              {{ companionStore.state.isConnected ? 'Lancement 1-clic direct dans Farming Simulator 25.' : 'Requis pour synchroniser automatiquement vos fichiers mods.' }}
            </p>
          </div>
        </div>

        <!-- Actionable onboarding link when disconnected -->
        <div v-if="!companionStore.state.isConnected" class="mt-4">
          <router-link to="/settings">
            <Button size="sm" variant="outline" class="w-full rounded-full text-xs">
              Configurer le compagnon
            </Button>
          </router-link>
        </div>

        <!-- Bottom Row: Refresh -->
        <div class="mt-6 pt-4 border-t border-surface-border flex items-center justify-end">
          <button
            type="button"
            @click="simulateRefresh"
            :disabled="isLoading"
            class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full border border-surface-border text-xs text-muted hover:text-foreground hover:border-foreground/30 transition-colors focus:outline-none"
            title="Rafraîchir l'état des serveurs"
          >
            <RotateCw :class="['w-3.5 h-3.5', isLoading ? 'animate-spin' : '']" />
            <span>Actualiser</span>
          </button>
        </div>
      </div>

    </div>

    <!-- Swiss Search and Filter Bar -->
    <div class="bg-surface border border-surface-border rounded-xl p-3 sm:p-4 flex flex-col md:flex-row items-stretch md:items-center justify-between gap-3 shadow-swiss-sm">
      <div class="flex-1 relative">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Rechercher par nom de ferme, adresse IP, carte..."
          class="w-full bg-surface-raised border border-surface-border rounded-lg px-3.5 py-2 pl-9 text-xs text-foreground focus:outline-none focus:border-primary font-sans placeholder:text-muted"
        />
        <Search class="w-4 h-4 absolute left-3 top-2.5 text-muted pointer-events-none" />
      </div>

      <!-- TLD / Zone filter pills -->
      <div class="flex flex-wrap items-center gap-1.5">
        <button
          type="button"
          v-for="tld in [
            { label: 'Tous', val: 'all' },
            { label: 'Europe (.eu)', val: '.eu' },
            { label: 'France (.fr)', val: '.fr' },
            { label: 'Global (.com)', val: '.com' },
          ]"
          :key="tld.val"
          @click="selectedTld = tld.val"
          :class="[
            'px-3 py-1.5 rounded-full text-xs font-semibold uppercase tracking-wider transition-all',
            selectedTld === tld.val
              ? 'bg-brand-dark text-white dark:bg-white dark:text-brand-dark shadow-sm'
              : 'bg-surface-raised text-muted hover:text-foreground border border-surface-border'
          ]"
        >
          {{ tld.label }}
        </button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading" class="grid grid-cols-1 md:grid-cols-2 gap-5">
      <div v-for="i in 4" :key="i" class="bg-surface border border-surface-border rounded-2xl p-6 space-y-4 shadow-swiss-sm">
        <div class="flex justify-between items-center">
          <GeometricSkeleton width="w-48" height="h-6" />
          <GeometricSkeleton width="w-24" height="h-6" />
        </div>
        <div class="grid grid-cols-3 gap-3">
          <GeometricSkeleton height="h-14" />
          <GeometricSkeleton height="h-14" />
          <GeometricSkeleton height="h-14" />
        </div>
      </div>
    </div>

    <!-- Server Cards Grid (Swiss Editorial Aesthetic) -->
    <div v-else-if="filteredServers.length > 0" class="grid grid-cols-1 md:grid-cols-2 gap-5">
      <div
        v-for="(server, idx) in filteredServers"
        :key="server.id"
        class="bg-surface border border-surface-border hover:border-primary rounded-2xl p-6 sm:p-7 transition-all flex flex-col justify-between shadow-swiss-sm relative group"
      >
        <!-- Top Editorial Meta Row -->
        <div>
          <div class="flex items-center justify-between pb-3 border-b border-surface-border">
            <div class="flex items-center gap-2 text-xs font-mono text-muted uppercase font-bold">
              <span>SERVEUR 0{{ idx + 1 }}</span>
              <span>•</span>
              <span class="text-primary font-bold">v{{ server.modpackVersion }}</span>
            </div>

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

          <!-- Server Name & Map -->
          <div class="mt-4">
            <div class="flex items-start justify-between gap-2">
              <h2 class="text-xl sm:text-2xl font-black font-display tracking-tight text-foreground uppercase">
                {{ server.name }}
              </h2>
              <span
                v-if="server.isOwner"
                class="px-2 py-0.5 bg-brand-dark text-white text-[10px] uppercase font-bold tracking-wider rounded-md shrink-0 dark:bg-purple-950 dark:text-purple-300"
              >
                Hôte
              </span>
            </div>

            <!-- Map and Address Details -->
            <div class="flex flex-wrap items-center gap-2 mt-2">
              <span class="px-2.5 py-0.5 rounded-full bg-surface-raised border border-surface-border text-xs font-semibold text-foreground/80">
                🗺️ {{ server.mapName }}
              </span>

              <button
                type="button"
                @click="copyAddress(server.address)"
                class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-surface-raised border border-surface-border text-xs font-mono text-muted hover:text-foreground transition-colors"
                title="Copier l'adresse IP"
              >
                <span>{{ server.address }}</span>
                <Copy class="w-3 h-3 text-muted" />
              </button>

              <button
                v-if="server.inviteCode"
                type="button"
                @click="copyInviteCode(server.inviteCode)"
                class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-primary/10 border border-primary/30 text-xs font-mono text-primary font-bold hover:bg-primary/20 transition-colors"
                title="Copier le code d'invitation"
              >
                <span>INVITE: {{ server.inviteCode }}</span>
                <Copy class="w-3 h-3" />
              </button>
            </div>
          </div>

          <!-- Metrics Row (Swiss Grid Numbers) -->
          <div class="grid grid-cols-3 gap-2 mt-5 p-3 rounded-xl bg-surface-raised border border-surface-border text-xs">
            <div>
              <div class="text-[10px] font-bold uppercase tracking-wider text-muted font-display">Joueurs</div>
              <div class="text-sm font-bold font-mono text-foreground mt-0.5">
                {{ server.currentPlayers }} / {{ server.maxPlayers }}
              </div>
            </div>

            <div>
              <div class="text-[10px] font-bold uppercase tracking-wider text-muted font-display">Mods</div>
              <div class="text-sm font-bold font-mono text-primary mt-0.5">
                {{ server.modCount }} mods
              </div>
            </div>

            <div>
              <div class="text-[10px] font-bold uppercase tracking-wider text-muted font-display">Taille</div>
              <div class="text-sm font-bold font-mono text-foreground mt-0.5">
                {{ formatBytes(server.totalSizeBytes) }}
              </div>
            </div>
          </div>
        </div>

        <!-- Bottom Action Buttons -->
        <div class="mt-6 pt-4 border-t border-surface-border flex flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <!-- Modpack Detail Link -->
            <router-link :to="`/servers/${server.id}`">
              <Button size="sm" variant="outline" class="rounded-full">
                <Layers class="w-3.5 h-3.5 text-primary" />
                Modpack
                <ArrowRight class="w-3 h-3 text-muted" />
              </Button>
            </router-link>

            <!-- Admin Link (Owner only) -->
            <router-link
              v-if="server.isOwner"
              :to="`/servers/${server.id}/admin`"
            >
              <Button size="sm" variant="secondary" class="rounded-full">
                <Shield class="w-3.5 h-3.5" />
                Gérer
              </Button>
            </router-link>
          </div>

          <!-- Join 1-Click Game Button -->
          <Button
            size="sm"
            :variant="server.status === 'synced' ? 'primary' : 'dark'"
            @click="joinServer(server)"
            class="rounded-full"
          >
            <Play class="w-3.5 h-3.5" />
            Lancer FS25
          </Button>
        </div>

      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="bg-surface border border-surface-border rounded-2xl p-12 text-center shadow-swiss-sm">
      <div class="w-12 h-12 rounded-full bg-surface-raised border border-surface-border flex items-center justify-center mx-auto text-muted mb-3">
        <ServerIcon class="w-6 h-6" />
      </div>
      <h3 class="text-base font-bold uppercase tracking-wider text-foreground font-display">
        Aucun serveur trouvé
      </h3>
      <p class="text-xs text-muted mt-1 font-sans">
        Modifiez vos filtres de recherche ou créez votre propre serveur multijoueur FS25.
      </p>
      <div class="mt-5">
        <Button size="sm" variant="primary" @click="showServerModal = true" class="rounded-full">
          <Plus class="w-3.5 h-3.5" />
          Ajouter un serveur
        </Button>
      </div>
    </div>

    <!-- Server Create / Join Modal -->
    <ServerActionModal
      :is-open="showServerModal"
      @close="showServerModal = false"
    />
  </div>
</template>
