<script setup lang="ts">
import { ref } from 'vue'
import { useCompanionStore } from '@/stores/companion'
import { useToastStore } from '@/stores/toast'
import {
  Download,
  Terminal,
  FolderOpen,
  Key,
  Copy,
  Check,
  RefreshCw,
  HardDrive,
  Radio,
} from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'

const companionStore = useCompanionStore()
const toastStore = useToastStore()

const inputModsPath = ref(companionStore.state.modsPath)
const apiToken = ref('flk_demo_sample_token_for_local_client')
const isCopied = ref(false)

function savePath() {
  companionStore.setModsPath(inputModsPath.value.trim())
}

function setPresetPath(os: 'windows' | 'linux') {
  if (os === 'windows') {
    inputModsPath.value = 'C:\\Users\\Utilisateur\\Documents\\My Games\\FarmingSimulator2025\\mods'
  } else {
    inputModsPath.value = '/home/user/.local/share/FarmingSimulator2025/mods'
  }
}

function copyToken() {
  navigator.clipboard.writeText(apiToken.value)
  isCopied.value = true
  toastStore.success('Jeton copié', 'Le token de liaison a été copié dans le presse-papier.')
  setTimeout(() => {
    isCopied.value = false
  }, 2500)
}

function regenerateToken() {
  apiToken.value = `flk_demo_${Math.random().toString(36).substring(2, 12)}${Math.random().toString(36).substring(2, 12)}`
  toastStore.info('Nouveau jeton généré', 'Pensez à mettre à jour la configuration de votre client Rust.')
}

function downloadBinary(os: 'windows' | 'linux') {
  const fileName = os === 'windows' ? 'farmlink-companion-v0.4.2-x86_64-pc-windows-msvc.exe' : 'farmlink-companion-v0.4.2-x86_64-unknown-linux-gnu.tar.gz'
  toastStore.success(
    'Téléchargement lancé',
    `Téléchargement du binaire Rust natif pour ${os === 'windows' ? 'Windows' : 'Linux'} : ${fileName}`
  )
}
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
    
    <!-- Header -->
    <div class="industrial-panel p-6 bg-surface flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 bg-primary/20 border border-primary/40 rounded-sm flex items-center justify-center text-primary">
            <Terminal class="w-5 h-5" />
          </div>
          <div>
            <h1 class="text-xl font-black uppercase tracking-wider text-foreground">
              Paramètres & Application Compagnon Rust
            </h1>
            <p class="text-xs text-muted font-mono mt-0.5">
              Client natif haute performance pour la synchronisation des mods locaux FS25
            </p>
          </div>
        </div>
      </div>

      <!-- Quick status indicator -->
      <Badge :variant="companionStore.state.isConnected ? 'synced' : 'missing'">
        <Radio class="w-3.5 h-3.5" />
        {{ companionStore.state.isConnected ? 'Client Rust connecté (port 45823)' : 'Client Rust non connecté' }}
      </Badge>
    </div>

    <!-- Section A: Téléchargement de l'App Compagnon Rust -->
    <div class="space-y-4">
      <div>
        <h2 class="text-sm font-bold uppercase tracking-wider text-foreground flex items-center gap-2">
          <Download class="w-4 h-4 text-primary" />
          1. Téléchargement du Client Rust Natif
        </h2>
        <p class="text-xs text-muted font-mono mt-1">
          Binaire compilé sans dépendance externe, consommant moins de 15 Mo de RAM avec support des liens IPC/WebSocket.
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Windows Binary Card -->
        <div class="industrial-panel p-6 bg-surface hover:border-gray-500 transition-colors flex flex-col justify-between">
          <div>
            <div class="flex items-center justify-between mb-3">
              <span class="text-xs font-mono px-2 py-0.5 bg-surface-raised border border-surface-border text-foreground">
                WINDOWS 10 / 11 (64-BIT)
              </span>
              <span class="text-xs font-mono text-muted">v0.4.2</span>
            </div>
            <h3 class="text-base font-bold text-foreground">
              FarmLink Companion pour Windows
            </h3>
            <p class="text-xs text-muted mt-2 leading-relaxed">
              Installeur autonome avec enregistrement automatique du protocole <code>farmlink://</code> pour rejoindre les serveurs en 1 clic.
            </p>
          </div>

          <div class="mt-6 pt-4 border-t border-surface-border flex items-center justify-between">
            <span class="text-xs font-mono text-muted">8.4 MB &bull; .exe</span>
            <Button size="sm" variant="primary" @click="downloadBinary('windows')">
              <Download class="w-4 h-4" />
              Télécharger pour Windows
            </Button>
          </div>
        </div>

        <!-- Linux Binary Card -->
        <div class="industrial-panel p-6 bg-surface hover:border-gray-500 transition-colors flex flex-col justify-between">
          <div>
            <div class="flex items-center justify-between mb-3">
              <span class="text-xs font-mono px-2 py-0.5 bg-surface-raised border border-surface-border text-foreground">
                LINUX / STEAM DECK / PROTON
              </span>
              <span class="text-xs font-mono text-muted">v0.4.2</span>
            </div>
            <h3 class="text-base font-bold text-foreground">
              FarmLink Companion pour Linux
            </h3>
            <p class="text-xs text-muted mt-2 leading-relaxed">
              Archive binaire statique (glibc / musl) et AppImage. Détecte automatiquement les préfixes Proton et Steam.
            </p>
          </div>

          <div class="mt-6 pt-4 border-t border-surface-border flex items-center justify-between">
            <span class="text-xs font-mono text-muted">6.9 MB &bull; tar.gz</span>
            <Button size="sm" variant="outline" @click="downloadBinary('linux')">
              <Download class="w-4 h-4" />
              Télécharger pour Linux
            </Button>
          </div>
        </div>
      </div>
    </div>

    <!-- Section B: Configuration du chemin local FS25 -->
    <div class="space-y-4">
      <div>
        <h2 class="text-sm font-bold uppercase tracking-wider text-foreground flex items-center gap-2">
          <FolderOpen class="w-4 h-4 text-primary" />
          2. Configuration du Dossier des Mods FS25
        </h2>
        <p class="text-xs text-muted font-mono mt-1">
          Emplacement où le client Rust télécharge, vérifie les sommes MD5 et synchronise vos archives .zip.
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
              <div class="text-foreground font-semibold mb-1">Linux / Steam Deck :</div>
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
            Chemin actuel utilisé par l'application compagnon :
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

    <!-- Section C: Clé d'API / Jeton de liaison -->
    <div class="space-y-4">
      <div>
        <h2 class="text-sm font-bold uppercase tracking-wider text-foreground flex items-center gap-2">
          <Key class="w-4 h-4 text-primary" />
          3. Clé d'API & Jeton de Liaison
        </h2>
        <p class="text-xs text-muted font-mono mt-1">
          Jeton cryptographique permettant à votre application Rust de s'authentifier auprès de votre compte FarmLink.
        </p>
      </div>

      <div class="industrial-panel p-6 bg-surface space-y-4">
        <div class="flex flex-col sm:flex-row items-center gap-3">
          <div class="w-full relative">
            <input
              :value="apiToken"
              readonly
              type="text"
              class="w-full bg-surface-raised border border-surface-border px-3 py-2 text-xs font-mono text-primary rounded-sm select-all focus:outline-none"
            />
          </div>

          <div class="flex items-center gap-2 w-full sm:w-auto">
            <Button size="sm" variant="secondary" @click="copyToken">
              <Check v-if="isCopied" class="w-4 h-4 text-primary" />
              <Copy v-else class="w-4 h-4" />
              {{ isCopied ? 'Copié !' : 'Copier' }}
            </Button>

            <Button size="sm" variant="outline" @click="regenerateToken">
              <RefreshCw class="w-4 h-4" />
              Régénérer
            </Button>
          </div>
        </div>

        <p class="text-[11px] text-muted font-mono">
          Dans votre application compagnon Rust, collez ce jeton dans le fichier <code class="text-foreground">farmlink.toml</code> ou dans l'écran de configuration initiale.
        </p>
      </div>
    </div>
  </div>
</template>
