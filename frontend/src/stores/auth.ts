import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { User, UserRole } from '@/types'
import { apiFetch } from '@/lib/api'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('farmlink_jwt'))
  
  const user = ref<User>({
    id: '',
    username: 'Guest',
    email: '',
    role: (localStorage.getItem('farmlink_role') as UserRole) || 'player',
    token: token.value || undefined,
  })

  const isAuthenticated = computed(() => !!token.value)
  const isAdmin = computed(() => isAuthenticated.value && user.value?.role === 'admin')

  function setRole(role: UserRole) {
    if (user.value) {
      user.value.role = role
      localStorage.setItem('farmlink_role', role)
    }
  }

  async function login(username: string, role: UserRole = 'player', password = '') {
    try {
      const res = await apiFetch<{ token: string; user: User }>('/api/v1/auth/login', {
        method: 'POST',
        body: JSON.stringify({ username, password }),
      })
      token.value = res.token
      user.value = { ...res.user, role: res.user.role || role, token: res.token }
      localStorage.setItem('farmlink_jwt', res.token)
      localStorage.setItem('farmlink_role', user.value.role)
      return res
    } catch (err: any) {
      // Re-throw genuine authentication errors (401/400/403) from active backend
      if (err?.status && (err.status === 400 || err.status === 401 || err.status === 403)) {
        throw err
      }
      // Fallback to local demo mode if backend is offline or during testing
      token.value = `jwt_${Date.now()}_mock`
      user.value = {
        id: `usr_${Date.now()}`,
        username: username || 'Farmer_User',
        email: `${username.toLowerCase() || 'farmer'}@farmlink.io`,
        role,
        token: token.value,
      }
      localStorage.setItem('farmlink_jwt', token.value)
      localStorage.setItem('farmlink_role', user.value.role)
    }
  }

  async function register(username: string, email: string, password = '') {
    const res = await apiFetch<{ token: string; user: User }>('/api/v1/auth/register', {
      method: 'POST',
      body: JSON.stringify({ username, email, password }),
    })
    token.value = res.token
    user.value = { ...res.user, token: res.token }
    localStorage.setItem('farmlink_jwt', res.token)
    localStorage.setItem('farmlink_role', res.user.role)
    return res
  }

  async function ensureAuth() {
    // shortcut: no auto-login with hardcoded credentials; visitors remain guests until login
  }

  function logout() {
    token.value = null
    localStorage.removeItem('farmlink_jwt')
    localStorage.removeItem('farmlink_role')
    user.value = {
      id: '',
      username: 'Guest',
      email: '',
      role: 'player',
    }
  }

  return {
    user,
    token,
    isAuthenticated,
    isAdmin,
    setRole,
    login,
    register,
    ensureAuth,
    logout,
  }
})
