import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { CompanionState } from '@/types'
import { useToastStore } from './toast'

export const useCompanionStore = defineStore('companion', () => {
  const toastStore = useToastStore()

  const state = ref<CompanionState>({
    isConnected: true, // Default to true for great out-of-the-box demo experience
    isConnecting: false,
    version: 'v0.4.2-rust',
    os: 'linux',
    modsPath: '/home/rauxde/.local/share/FarmingSimulator2025/mods',
    lastSyncAt: new Date().toISOString(),
    syncInProgress: false,
    syncProgress: 0,
    currentSyncMod: null,
    simulationMode: true,
  })

  let socket: WebSocket | null = null

  function toggleConnection() {
    state.value.isConnected = !state.value.isConnected
    if (state.value.isConnected) {
      toastStore.success(
        'App Compagnon connectée',
        `Liaison établie avec le client Rust (${state.value.version})`
      )
    } else {
      toastStore.warning(
        'App Compagnon déconnectée',
        'Le client Rust n\'est plus joignable sur le port local 45823'
      )
    }
  }

  function setModsPath(path: string) {
    state.value.modsPath = path
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({ type: 'set_mods_path', path }))
    }
    toastStore.success('Chemin mis à jour', `Dossier des mods configuré : ${path}`)
  }

  async function syncAllMods(modNames: string[] = ['FS25_JohnDeere_7R.zip', 'FS25_Courseplay.zip', 'FS25_ClaasLexion8900.zip']) {
    if (state.value.syncInProgress) return
    if (!state.value.isConnected) {
      toastStore.error(
        'Synchronisation impossible',
        'L\'application compagnon Rust doit être connectée pour synchroniser vos mods locaux.'
      )
      return
    }

    // If connected to real Rust companion via WebSocket, trigger native sync
    if (socket && socket.readyState === WebSocket.OPEN) {
      state.value.syncInProgress = true
      state.value.syncProgress = 0
      toastStore.info('Synchronisation démarrée', `Transfert et vérification SHA-256 via le client Rust...`)
      socket.send(JSON.stringify({
        type: 'sync_mods',
        mod_names: modNames,
      }))
      return
    }

    state.value.syncInProgress = true
    state.value.syncProgress = 0
    toastStore.info('Synchronisation démarrée', `Vérification et transfert de ${modNames.length} mods...`)

    for (let i = 0; i < modNames.length; i++) {
      state.value.currentSyncMod = modNames[i]
      state.value.syncProgress = Math.round(((i + 0.3) / modNames.length) * 100)
      await new Promise((r) => setTimeout(r, 650))
      state.value.syncProgress = Math.round(((i + 1) / modNames.length) * 100)
    }

    state.value.syncInProgress = false
    state.value.currentSyncMod = null
    state.value.lastSyncAt = new Date().toISOString()
    toastStore.success('Synchronisation terminée', 'Tous les mods sont à jour dans votre répertoire local FS25.')
  }

  function tryRealWebSocketConnect() {
    if (socket) {
      socket.close()
      socket = null
    }
    state.value.isConnecting = true
    try {
      socket = new WebSocket('ws://127.0.0.1:45823/ws')
      socket.onopen = () => {
        state.value.isConnected = true
        state.value.isConnecting = false
        state.value.simulationMode = false
        toastStore.success('Liaison Rust IPC active', 'Connecté au client local natif.')
      }

      socket.onmessage = (event) => {
        try {
          const msg = JSON.parse(event.data)
          if (msg.type === 'state') {
            state.value.isConnected = msg.is_connected
            state.value.version = msg.version
            state.value.os = msg.os
            state.value.modsPath = msg.mods_path
            state.value.syncInProgress = msg.sync_in_progress
            state.value.syncProgress = msg.sync_progress
            state.value.currentSyncMod = msg.current_sync_mod
          } else if (msg.type === 'sync_started') {
            state.value.syncInProgress = true
            state.value.syncProgress = 0
          } else if (msg.type === 'sync_progress') {
            state.value.syncInProgress = true
            state.value.syncProgress = msg.overall_progress
            state.value.currentSyncMod = msg.file_name
          } else if (msg.type === 'sync_completed') {
            state.value.syncInProgress = false
            state.value.currentSyncMod = null
            state.value.lastSyncAt = msg.last_sync_at
            if (msg.errors && msg.errors.length > 0) {
              toastStore.warning('Synchronisation terminée avec alertes', `${msg.synced_count} mod(s) à jour, ${msg.failed_count} échec(s).`)
            } else {
              toastStore.success('Synchronisation terminée', `${msg.synced_count} mod(s) vérifiés et synchronisés.`)
            }
          } else if (msg.type === 'scan_result') {
            toastStore.info('Scan terminé', `${msg.total_mods} mods locaux analysés (${(msg.total_size_bytes / (1024 * 1024)).toFixed(1)} MB).`)
          } else if (msg.type === 'game_launched') {
            if (msg.success) {
              toastStore.success('Jeu lancé', msg.message)
            } else {
              toastStore.error('Erreur lancement jeu', msg.message)
            }
          } else if (msg.type === 'error') {
            toastStore.error('Erreur Client Rust', msg.message)
          }
        } catch {
          // ignore invalid message
        }
      }

      socket.onerror = () => {
        state.value.isConnecting = false
        // Keep simulated mode active so user has functioning UI
      }
      socket.onclose = () => {
        if (!state.value.simulationMode) {
          state.value.isConnected = false
        }
      }
    } catch {
      state.value.isConnecting = false
    }
  }

  return {
    state,
    toggleConnection,
    setModsPath,
    syncAllMods,
    tryRealWebSocketConnect,
  }
})
