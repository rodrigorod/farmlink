import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { CompanionState } from '@/types'
import { useToastStore } from './toast'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export const useCompanionStore = defineStore('companion', () => {
  const toastStore = useToastStore()

  const state = ref<CompanionState>({
    isConnected: true, // Default to connected in Tauri or simulated demo
    isConnecting: false,
    version: 'v0.4.2-rust',
    os: 'linux',
    modsPath: '/home/rauxde/.local/share/FarmingSimulator2025/mods',
    lastSyncAt: new Date().toISOString(),
    syncInProgress: false,
    syncProgress: 0,
    currentSyncMod: null,
    simulationMode: !isTauri(),
  })

  let unlisteners: UnlistenFn[] = []

  function toggleConnection() {
    state.value.isConnected = !state.value.isConnected
    if (state.value.isConnected) {
      toastStore.success(
        'App Compagnon connectée',
        `Liaison établie avec le client natif (${state.value.version})`
      )
    } else {
      toastStore.warning(
        'App Compagnon déconnectée',
        'Liaison IPC native suspendue'
      )
    }
  }

  async function setModsPath(path: string) {
    state.value.modsPath = path
    if (isTauri()) {
      try {
        await invoke('set_mods_path', { path })
        toastStore.success('Chemin mis à jour', `Dossier des mods configuré : ${path}`)
      } catch (e) {
        toastStore.error('Erreur', String(e))
      }
    } else {
      toastStore.success('Chemin mis à jour', `Dossier des mods configuré : ${path}`)
    }
  }

  async function scanMods(modsPath?: string) {
    if (isTauri()) {
      try {
        const result = await invoke<any[]>('scan_mods', { modsPath })
        toastStore.info('Scan terminé', `${result.length} mods locaux analysés via le backend natif.`)
        return result
      } catch (e) {
        toastStore.error('Erreur de scan', String(e))
        return []
      }
    }
    return []
  }

  async function launchGame(target = 'auto') {
    if (isTauri()) {
      try {
        const res = await invoke<any>('launch_game', { target })
        const message = typeof res === 'string' ? res : res?.message || 'Farming Simulator 25 lancé'
        const isSuccess = typeof res === 'string' ? true : res?.success !== false
        if (isSuccess) {
          toastStore.success('Jeu lancé', message)
          return { success: true, message }
        } else {
          toastStore.error('Erreur lancement', message)
          return { success: false, message }
        }
      } catch (e) {
        toastStore.error('Erreur', String(e))
        return { success: false, message: String(e) }
      }
    } else {
      toastStore.success('Jeu simulé', 'Simulation du lancement de Farming Simulator 25')
      return { success: true, message: 'Simulation du lancement FS25' }
    }
  }

  async function syncAllMods(
    modNames: string[] = ['FS25_JohnDeere_7R.zip', 'FS25_Courseplay.zip', 'FS25_ClaasLexion8900.zip'],
    serverId = 'server-1'
  ) {
    if (state.value.syncInProgress) return
    if (!state.value.isConnected) {
      toastStore.error(
        'Synchronisation impossible',
        'L\'application compagnon Rust doit être connectée pour synchroniser vos mods locaux.'
      )
      return
    }

    if (isTauri()) {
      state.value.syncInProgress = true
      state.value.syncProgress = 0
      toastStore.info('Synchronisation démarrée', 'Transfert et vérification SHA-256 via le client natif...')
      try {
        await invoke('sync_server', { serverId, modNames })
      } catch (e) {
        state.value.syncInProgress = false
        state.value.currentSyncMod = null
        toastStore.error('Erreur de synchronisation', String(e))
      }
      return
    }

    // Simulated fallback for browser demo or vitest
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

  async function initTauriListeners() {
    if (!isTauri()) {
      return
    }

    state.value.isConnected = true
    state.value.simulationMode = false

    try {
      const initialState = await invoke<any>('get_companion_state')
      if (initialState) {
        state.value.isConnected = initialState.is_connected
        state.value.version = initialState.version
        state.value.os = initialState.os
        state.value.modsPath = initialState.mods_path
        state.value.syncInProgress = initialState.sync_in_progress
        state.value.syncProgress = initialState.sync_progress
        state.value.currentSyncMod = initialState.current_sync_mod
      }
    } catch (e) {
      console.warn('Impossible de récupérer l\'état initial du client Rust:', e)
    }

    // Clean up previous listeners if any
    for (const unlisten of unlisteners) {
      unlisten()
    }
    unlisteners = []

    try {
      const unlistenProgress = await listen('sync_progress', (event) => {
        const msg = event.payload as any
        state.value.syncInProgress = true
        state.value.syncProgress = msg.overall_progress
        state.value.currentSyncMod = msg.file_name
      })
      unlisteners.push(unlistenProgress)

      const unlistenCompleted = await listen('sync_completed', (event) => {
        const msg = event.payload as any
        state.value.syncInProgress = false
        state.value.currentSyncMod = null
        state.value.lastSyncAt = msg.last_sync_at
        if (msg.errors && msg.errors.length > 0) {
          toastStore.warning('Synchronisation terminée avec alertes', `${msg.synced_count} mod(s) à jour, ${msg.failed_count} échec(s).`)
        } else {
          toastStore.success('Synchronisation terminée', `${msg.synced_count} mod(s) vérifiés et synchronisés.`)
        }
      })
      unlisteners.push(unlistenCompleted)

      const unlistenScan = await listen('scan_result', (event) => {
        const msg = event.payload as any
        toastStore.info('Scan terminé', `${msg.total_mods} mods locaux analysés (${(msg.total_size_bytes / (1024 * 1024)).toFixed(1)} MB).`)
      })
      unlisteners.push(unlistenScan)

      const unlistenError = await listen('error', (event) => {
        const msg = event.payload as any
        toastStore.error('Erreur Client Rust', msg.message)
      })
      unlisteners.push(unlistenError)
    } catch (e) {
      console.warn('Erreur lors de l\'enregistrement des listeners Tauri:', e)
    }
  }

  // Backward compatibility alias for reconnection trigger
  async function tryRealWebSocketConnect() {
    await initTauriListeners()
  }

  return {
    state,
    toggleConnection,
    setModsPath,
    syncAllMods,
    scanMods,
    launchGame,
    initTauriListeners,
    tryRealWebSocketConnect,
  }
})
