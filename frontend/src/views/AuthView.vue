<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useToastStore } from '@/stores/toast'
import type { UserRole } from '@/types'
import {
  Tractor,
  KeyRound,
  ShieldCheck,
  UserCheck,
  CheckCircle2,
  Lock,
} from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'

const router = useRouter()
const authStore = useAuthStore()
const toastStore = useToastStore()

const isRegister = ref(false)
const username = ref('Farmer_Alex')
const email = ref('alex@farmlink.io')
const password = ref('password123')
const selectedRole = ref<UserRole>('admin')

async function handleSubmit() {
  if (!username.value.trim()) {
    toastStore.warning('Champ requis', 'Veuillez saisir votre pseudonyme joueur FS25.')
    return
  }

  try {
    if (isRegister.value) {
      await authStore.register(username.value, email.value, password.value)
    } else {
      await authStore.login(username.value, selectedRole.value, password.value)
    }
    toastStore.success(
      isRegister.value ? 'Compte créé avec succès' : 'Connexion réussie',
      `Bienvenue ${authStore.user?.username} (Rôle : ${authStore.user?.role === 'admin' ? 'Administrateur Serveur' : 'Joueur Co-op'})`
    )
    router.push('/dashboard')
  } catch (err: any) {
    toastStore.error(
      'Erreur d\'authentification',
      err?.message || 'Impossible de se connecter au serveur backend.'
    )
  }
}
</script>

<template>
  <div class="min-h-[calc(100vh-4rem)] flex items-center justify-center p-4 bg-tech-grid">
    <div class="max-w-md w-full industrial-panel p-8 shadow-2xl relative">
      <!-- Cockpit corner rivet decorations -->
      <div class="absolute top-2 left-2 w-1.5 h-1.5 bg-surface-border"></div>
      <div class="absolute top-2 right-2 w-1.5 h-1.5 bg-surface-border"></div>
      <div class="absolute bottom-2 left-2 w-1.5 h-1.5 bg-surface-border"></div>
      <div class="absolute bottom-2 right-2 w-1.5 h-1.5 bg-surface-border"></div>

      <!-- Header -->
      <div class="text-center mb-6">
        <div class="inline-flex p-3 bg-primary/10 border border-primary/30 rounded-sm text-primary mb-3">
          <Tractor class="w-8 h-8" />
        </div>
        <h2 class="text-xl font-black uppercase tracking-wider text-foreground">
          {{ isRegister ? 'Créer un profil FarmLink' : 'Authentification Cockpit' }}
        </h2>
        <p class="text-xs text-muted mt-1 font-mono">
          FS25 Dedicated Server & Modpack Sync System
        </p>
      </div>

      <!-- Auth Form -->
      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div>
          <label class="block text-xs font-semibold uppercase tracking-wider text-muted mb-1">
            Pseudonyme FS25
          </label>
          <div class="relative">
            <input
              v-model="username"
              type="text"
              required
              class="w-full bg-surface-raised border border-surface-border rounded-sm px-3 py-2 text-sm text-foreground focus:outline-none focus:border-primary font-mono"
              placeholder="ex: AgriPro_Alex"
            />
          </div>
        </div>

        <div v-if="isRegister">
          <label class="block text-xs font-semibold uppercase tracking-wider text-muted mb-1">
            Adresse E-mail
          </label>
          <input
            v-model="email"
            type="email"
            required
            class="w-full bg-surface-raised border border-surface-border rounded-sm px-3 py-2 text-sm text-foreground focus:outline-none focus:border-primary font-mono"
            placeholder="alex@farmlink.io"
          />
        </div>

        <div>
          <label class="block text-xs font-semibold uppercase tracking-wider text-muted mb-1">
            Mot de passe / Token
          </label>
          <div class="relative">
            <input
              v-model="password"
              type="password"
              required
              class="w-full bg-surface-raised border border-surface-border rounded-sm px-3 py-2 text-sm text-foreground focus:outline-none focus:border-primary font-mono"
              placeholder="••••••••••••"
            />
            <Lock class="w-4 h-4 absolute right-3 top-2.5 text-muted pointer-events-none" />
          </div>
        </div>

        <!-- Role Selector (Cahier des charges: Gestion des rôles Admin vs Joueur) -->
        <div>
          <label class="block text-xs font-semibold uppercase tracking-wider text-muted mb-2">
            Rôle initial sur la plateforme
          </label>
          <div class="grid grid-cols-2 gap-2">
            <button
              type="button"
              @click="selectedRole = 'player'"
              :class="[
                'p-3 border text-left rounded-sm transition-all',
                selectedRole === 'player'
                  ? 'bg-primary/10 border-primary text-foreground'
                  : 'bg-surface-raised border-surface-border text-muted hover:border-gray-500',
              ]"
            >
              <div class="flex items-center justify-between mb-1">
                <UserCheck class="w-4 h-4 text-emerald-400" />
                <CheckCircle2 v-if="selectedRole === 'player'" class="w-3.5 h-3.5 text-primary" />
              </div>
              <div class="text-xs font-bold uppercase tracking-wider">Joueur Co-op</div>
              <div class="text-[10px] text-muted mt-0.5">Téléchargement & synchronisation</div>
            </button>

            <button
              type="button"
              @click="selectedRole = 'admin'"
              :class="[
                'p-3 border text-left rounded-sm transition-all',
                selectedRole === 'admin'
                  ? 'bg-purple-950/40 border-purple-600 text-foreground'
                  : 'bg-surface-raised border-surface-border text-muted hover:border-gray-500',
              ]"
            >
              <div class="flex items-center justify-between mb-1">
                <ShieldCheck class="w-4 h-4 text-purple-400" />
                <CheckCircle2 v-if="selectedRole === 'admin'" class="w-3.5 h-3.5 text-purple-400" />
              </div>
              <div class="text-xs font-bold uppercase tracking-wider">Admin Serveur</div>
              <div class="text-[10px] text-muted mt-0.5">Upload .zip & gestion manifeste</div>
            </button>
          </div>
        </div>

        <!-- Token Information Notice -->
        <div class="p-3 bg-surface-raised border border-surface-border text-[11px] text-muted flex items-start gap-2">
          <KeyRound class="w-4 h-4 text-primary shrink-0 mt-0.5" />
          <span>
            Authentification par jeton JWT sécurisé émis par l'API Backend Go. Utilisable pour lier l'app compagnon Rust.
          </span>
        </div>

        <!-- Submit Button -->
        <Button type="submit" variant="primary" class="w-full mt-2">
          {{ isRegister ? 'Créer le compte' : 'Ouvrir la session' }}
        </Button>
      </form>

      <!-- Toggle Mode -->
      <div class="mt-6 pt-4 border-t border-surface-border text-center">
        <button
          type="button"
          @click="isRegister = !isRegister"
          class="text-xs text-primary hover:underline font-mono"
        >
          {{ isRegister ? 'Déjà un compte ? Connectez-vous' : 'Pas encore de compte ? Enregistrer un joueur' }}
        </button>
      </div>
    </div>
  </div>
</template>
