import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useAuthStore } from './auth'
import { useServersStore } from './servers'
import { useCompanionStore } from './companion'

const storage: Record<string, string> = {}
if (typeof globalThis.localStorage === 'undefined') {
  globalThis.localStorage = {
    getItem: (k: string) => storage[k] ?? null,
    setItem: (k: string, v: string) => { storage[k] = String(v) },
    removeItem: (k: string) => { delete storage[k] },
    clear: () => { Object.keys(storage).forEach((k) => delete storage[k]) },
    length: 0,
    key: () => null,
  }
}

describe('stores logic', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
  })

  it('logout removes jwt and role and resets player state', async () => {
    const authStore = useAuthStore()
    await authStore.login('AdminUser', 'admin')
    expect(authStore.isAuthenticated).toBe(true)
    expect(authStore.isAdmin).toBe(true)

    authStore.logout()
    expect(authStore.isAuthenticated).toBe(false)
    expect(authStore.user.role).toBe('player')
    expect(localStorage.getItem('farmlink_jwt')).toBeNull()
    expect(localStorage.getItem('farmlink_role')).toBeNull()
  })

  it('accepts uppercase .ZIP archives for mod upload', async () => {
    const serversStore = useServersStore()
    const serverId = serversStore.servers[0].id
    const countBefore = serversStore.getModsForServer(serverId).value.length

    const success = await serversStore.uploadModZip(serverId, {
      name: 'FS25_TEST_MOD.ZIP',
      size: 15000000,
    })

    expect(success).toBe(true)
    expect(serversStore.getModsForServer(serverId).value.length).toBe(countBefore + 1)
  })

  it('bumping modpack version marks server status as update_required', () => {
    const serversStore = useServersStore()
    const serverId = serversStore.servers[0].id

    serversStore.bumpModpackVersion(serverId, 'v2.5.0')
    const server = serversStore.getServerById(serverId).value
    expect(server?.modpackVersion).toBe('v2.5.0')
    expect(server?.status).toBe('update_required')
  })

  it('companion sync blocks re-entrant concurrent execution', async () => {
    const companionStore = useCompanionStore()
    companionStore.state.isConnected = true
    companionStore.state.syncInProgress = true

    // Second call should return early without altering state
    await companionStore.syncAllMods(['test.zip'])
    expect(companionStore.state.syncProgress).toBe(0)
  })

  it('companion setModsPath and launchGame fallback gracefully in browser environment', async () => {
    const companionStore = useCompanionStore()
    await companionStore.setModsPath('/custom/fs25/mods')
    expect(companionStore.state.modsPath).toBe('/custom/fs25/mods')

    const res = await companionStore.launchGame('farmlink://join?server=s1')
    expect(res.success).toBe(true)

    const scan = await companionStore.scanMods()
    expect(Array.isArray(scan)).toBe(true)
  })

  it('filters servers by name, address and TLD zone', () => {
    const serversStore = useServersStore()
    const filter = (q: string, tld = 'all') => {
      const term = q.trim().toLowerCase()
      return serversStore.servers.filter((s) => {
        const addr = s.address.toLowerCase()
        const matchesQ = !term || s.name.toLowerCase().includes(term) || addr.includes(term)
        const matchesTld = tld === 'all' || addr.includes(tld.toLowerCase())
        return matchesQ && matchesTld
      })
    }

    expect(filter('Zielonka').length).toBe(1)
    expect(filter('', '.eu').length).toBe(3)
    expect(filter('', '.fr').length).toBe(0)
    expect(filter('Riverbend', '.eu').length).toBe(1)
  })

  it('creates a new server with owner rights and invite code', async () => {
    const serversStore = useServersStore()
    const countBefore = serversStore.servers.length

    const created = await serversStore.createServer({
      name: 'Ferme des Vallons',
      ipAddress: '192.168.1.120',
      port: 10823,
      mapName: 'Zielonka (Pologne)',
    })

    expect(created).not.toBeNull()
    expect(created?.name).toBe('Ferme des Vallons')
    expect(created?.isOwner).toBe(true)
    expect(created?.inviteCode).toBeDefined()
    expect(created?.inviteCode?.startsWith('FLK-')).toBe(true)
    expect(serversStore.servers.length).toBe(countBefore + 1)
  })

  it('joins an existing server using an invite code', async () => {
    const serversStore = useServersStore()
    const countBefore = serversStore.servers.length

    const joined = await serversStore.joinServerByCode('COMMUNITY-2025')
    expect(joined).not.toBeNull()
    expect(joined?.isOwner).toBe(false)
    expect(joined?.inviteCode).toBe('COMMUNITY-2025')
    expect(serversStore.servers.length).toBe(countBefore + 1)
  })
})
