import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

import DashboardView from '@/views/DashboardView.vue'
import ServerDetailView from '@/views/ServerDetailView.vue'
import ServerAdminView from '@/views/ServerAdminView.vue'
import SettingsView from '@/views/SettingsView.vue'
import AuthView from '@/views/AuthView.vue'

const routes = [
  {
    path: '/',
    redirect: '/dashboard',
  },
  {
    path: '/auth',
    name: 'auth',
    component: AuthView,
    meta: { public: true },
  },
  {
    path: '/dashboard',
    name: 'dashboard',
    component: DashboardView,
  },
  {
    path: '/servers/:id',
    name: 'server-detail',
    component: ServerDetailView,
  },
  {
    path: '/servers/:id/admin',
    name: 'server-admin',
    component: ServerAdminView,
    meta: { requiresAdmin: true },
  },
  {
    path: '/settings',
    name: 'settings',
    component: SettingsView,
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/dashboard',
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior() {
    return { top: 0 }
  },
})

router.beforeEach((to, _from, next) => {
  const authStore = useAuthStore()
  if (!to.meta.public && !authStore.isAuthenticated) {
    next('/auth')
  } else if (to.meta.requiresAdmin && !authStore.isAdmin) {
    next('/dashboard')
  } else {
    next()
  }
})

export default router
