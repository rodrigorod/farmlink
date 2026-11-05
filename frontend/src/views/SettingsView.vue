<script setup lang="ts">
import { ref } from 'vue'
import { useCompanionStore } from '@/stores/companion'
import {
  FolderOpen,
  HardDrive,
  Cpu,
  RefreshCw,
  Sliders,
  CheckCircle2,
} from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'

const companionStore = useCompanionStore()
const inputModsPath = ref(companionStore.state.modsPath)
const isScanning = ref(false)

async function savePath() {
  await companionStore.setModsPath(inputModsPath.value.trim())
}

function setPresetPath(os: 'windows' | 'linux') {
  if (os === 'windows') {
    inputModsPath.value = 'C:\\Users\\Utilisateur\\Documents\\My Games\\FarmingSimulator2025\\mods'
  } else {
    inputModsPath.value = '/home/user/.local/share/FarmingSimulator2025/mods'
  }
}

async function triggerScan() {
  isScanning.value = true
  try {
    await companionStore.scanMods(inputModsPath.value.trim())
  } finally {
    isScanning.value = false
  }
}
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
    
    <!-- Header -->
    <div class="industrial-panel p-6 bg-surface flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 bg-primary/20 border border-primary/40 rounded-sm flex items-center justify-center text-primary">
            <Sliders class="w-5 h-5" />
          </div>
          <div>
            <h1 class="text-xl font-black uppercase tracking-wider text-foreground">
              Paramètres de l'Application
            </h1>
            <p class="text-xs text-muted font-mono mt-0.5">
              Configuration du répertoire de jeu local et statut du moteur natif Tauri IPC
            </p>
          </div>
        </div>
      </div>

      <!-- Quick status indicator -->
      <Badge variant="synced">
        <CheckCircle2 class="w-3.5 h-3.5" />
        Moteur Desktop Natif Actif
      </Badge>
    </div>

    <!-- Section 1: Configuration du chemin local FS25 -->
    <div class="space-y-4">
      <div>
        <h2 class="text-sm font-bold uppercase tracking-wider text-foreground flex items-center gap-2">
          <FolderOpen class="w-4 h-4 text-primary" />
          1. Répertoire des Mods Farming Simulator 25
        </h2>
        <p class="text-xs text-muted font-mono mt-1">
          Emplacement où le moteur Rust synchronise vos archives .zip et effectue les vérifications cryptographiques SHA-256.
        </p>
      </div>

      <div class="industrial-panel p-6 bg-surface space-y-4">
        <!-- Visual guide -->
        <div class="p-4 bg-surface-raised border border-surface-border text-xs space-y-2">
          <div class="font-bold text-foreground uppercase tracking-wider flex items-center gap-2">
            <HardDrive class="w-4 h-4 text-primary" />
            Chemins par défaut selon votre environnement :
          </div>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-muted font-mono text-[11px] mt-2">
            <div class="p-2 bg-surface border border-surface-border rounded-sm">
              <div class="text-foreground font-semibold mb-1">Windows standard :</div>
              <code class="text-primary break-all">
                C:\Users\&lt;Nom&gt;\Documents\My Games\FarmingSimulator2025\mods\
              </code>
              <button
                type="button"
                @click="setPresetPath('windows')"
                class="block text-[10px] text-muted hover:text-foreground mt-1 underline"
              >
                Appliquer ce modèle
              </button>
            </div>

            <div class="p-2 bg-surface border border-surface-border rounded-sm">
              <div class="text-foreground font-semibold mb-1">Linux / Steam Deck / Proton :</div>
              <code class="text-primary break-all">
                ~/.local/share/FarmingSimulator2025/mods/
              </code>
              <button
                type="button"
                @click="setPresetPath('linux')"
                class="block text-[10px] text-muted hover:text-foreground mt-1 underline"
              >
                Appliquer ce modèle
              </button>
            </div>
          </div>
        </div>

        <!-- Path input field -->
        <div class="space-y-2">
          <label class="block text-xs font-semibold uppercase tracking-wider text-muted">
            Chemin actuel configuré :
          </label>
          <div class="flex flex-col sm:flex-row gap-2">
            <input
              v-model="inputModsPath"
              type="text"
              class="flex-1 bg-surface-raised border border-surface-border px-3 py-2 text-xs font-mono text-foreground focus:outline-none focus:border-primary rounded-sm"
              placeholder="/chemin/vers/FarmingSimulator2025/mods"
            />
            <Button size="sm" variant="primary" @click="savePath">
              Enregistrer le chemin
            </Button>
          </div>
        </div>
      </div>
    </div>

    <!-- Section 2: Analyse & Indexation locale des mods -->
    <div class="space-y-4">
      <div>
        <h2 class="text-sm font-bold uppercase tracking-wider text-foreground flex items-center gap-2">
          <RefreshCw class="w-4 h-4 text-primary" />
          2. Indexation & Vérification de l'Intégrité
        </h2>
        <p class="text-xs text-muted font-mono mt-1">
          Force l'analyse des archives du dossier local pour extraire les métadonnées modDesc.xml et recalculer le cache SHA-256.
        </p>
      </div>

      <div class="industrial-panel p-6 bg-surface flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div class="text-xs font-bold text-foreground">
            Scan manuel des mods locaux
          </div>
          <div class="text-xs text-muted font-mono mt-0.5">
            Dernière synchronisation : {{ companionStore.state.lastSyncAt ? new Date(companionStore.state.lastSyncAt).toLocaleString('fr-FR') : 'Jamais' }}
          </div>
        </div>

        <Button
          size="sm"
          variant="secondary"
          @click="triggerScan"
          :disabled="isScanning"
        >
          <RefreshCw :class="['w-3.5 h-3.5', isScanning ? 'animate-spin' : '']" />
          {{ isScanning ? 'Scan en cours...' : 'Lancer le scan des mods' }}
        </Button>
      </div>
    </div>

    <!-- Section 3: Diagnostic & Environnement Système -->
    <div class="space-y-4">
      <div>
        <h2 class="text-sm font-bold uppercase tracking-wider text-foreground flex items-center gap-2">
          <Cpu class="w-4 h-4 text-primary" />
          3. Environnement & Spécifications Techniques
        </h2>
        <p class="text-xs text-muted font-mono mt-1">
          Informations matérielles et versioning du binaire autonome FarmLink
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="industrial-panel p-5 bg-surface">
          <div class="text-xs font-semibold text-muted uppercase font-mono mb-1">Version FarmLink</div>
          <div class="text-lg font-bold font-mono text-primary">{{ companionStore.state.version }}</div>
          <div class="text-[11px] text-muted font-mono mt-1">Tauri Native IPC v2</div>
        </div>

        <div class="industrial-panel p-5 bg-surface">
          <div class="text-xs font-semibold text-muted uppercase font-mono mb-1">Système d'Exploitation</div>
          <div class="text-lg font-bold font-mono uppercase text-foreground">{{ companionStore.state.os }} x64</div>
          <div class="text-[11px] text-muted font-mono mt-1">Détection dynamique OS</div>
        </div>

        <div class="industrial-panel p-5 bg-surface">
          <div class="text-xs font-semibold text-muted uppercase font-mono mb-1">Architecture IPC</div>
          <div class="text-lg font-bold font-mono text-emerald-400">Processus Isolé</div>
          <div class="text-[11px] text-muted font-mono mt-1">Zero-port réseau (sans WebSocket)</div>
        </div>
      </div>
    </div>
  </div>
</template>
