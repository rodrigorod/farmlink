import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { User, UserRole } from '@/types'
import { apiFetch } from '@/lib/api'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('farmlink_jwt'))
  
  const user = ref<User>({
    id: 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    username: 'Farmer_Alex',
    email: 'alex@farmlink.io',
    role: (localStorage.getItem('farmlink_role') as UserRole) || 'admin',
    token: token.value || undefined,
  })

  const isAuthenticated = computed(() => !!token.value)
  const isAdmin = computed(() => user.value?.role === 'admin')

  function setRole(role: UserRole) {
    if (user.value) {
      user.value.role = role
      localStorage.setItem('farmlink_role', role)
    }
  }

  async function login(username: string, role: UserRole = 'player', password = 'password123') {
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
    } catch {
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

  async function register(username: string, email: string, password = 'password123') {
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
    if (!token.value || token.value.includes('mock')) {
      // Auto-login with default seeded database admin
      try {
        await login('Farmer_Alex', 'admin', 'password123')
      } catch {
        // Silently keep fallback if backend starting
      }
    }
  }

  function logout() {
    token.value = null
    localStorage.removeItem('farmlink_jwt')
    localStorage.removeItem('farmlink_role')
    user.value.role = 'player'
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
