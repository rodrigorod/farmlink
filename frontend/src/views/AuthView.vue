<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useToastStore } from '@/stores/toast'
import { currentTheme, toggleTheme } from '@/lib/theme'
import type { UserRole } from '@/types'
import {
  ShieldCheck,
  UserCheck,
  CheckCircle2,
  Lock,
  Sun,
  Moon,
  ArrowRight,
} from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'

const router = useRouter()
const authStore = useAuthStore()
const toastStore = useToastStore()

const isRegister = ref(false)
const username = ref('')
const email = ref('')
const password = ref('')
const selectedRole = ref<UserRole>('player')

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
  <div class="min-h-screen flex flex-col justify-center items-center p-4 sm:p-6 bg-background relative">
    
    <!-- Top Bar with Theme Toggle -->
    <div class="absolute top-6 right-6">
      <button
        type="button"
        @click="toggleTheme"
        :title="currentTheme === 'dark' ? 'Activer le mode clair' : 'Activer le mode sombre'"
        class="w-9 h-9 rounded-full bg-surface border border-surface-border flex items-center justify-center text-muted hover:text-foreground transition-colors shadow-swiss-sm"
      >
        <Sun v-if="currentTheme === 'dark'" class="w-4 h-4 text-amber-400" />
        <Moon v-else class="w-4 h-4 text-slate-700" />
      </button>
    </div>

    <!-- Editorial Auth Card -->
    <div class="max-w-md w-full bg-surface border border-surface-border rounded-3xl p-8 sm:p-10 shadow-swiss-lg relative">
      
      <!-- Top Brand Logo -->
      <div class="text-center mb-6">
        <img
          src="/logo_readme.png"
          alt="FarmLink FS25"
          class="h-24 sm:h-28 w-auto mx-auto object-contain transition-transform hover:scale-105"
        />
      </div>

      <!-- Toggle Tab: Connexion vs Création -->
      <div class="grid grid-cols-2 p-1 bg-surface-raised border border-surface-border rounded-xl mb-6">
        <button
          type="button"
          @click="isRegister = false"
          :class="[
            'py-2 text-xs font-bold uppercase tracking-wider rounded-lg transition-all font-display',
            !isRegister
              ? 'bg-surface text-foreground shadow-swiss-sm'
              : 'text-muted hover:text-foreground'
          ]"
        >
          Connexion
        </button>
        <button
          type="button"
          @click="isRegister = true"
          :class="[
            'py-2 text-xs font-bold uppercase tracking-wider rounded-lg transition-all font-display',
            isRegister
              ? 'bg-surface text-foreground shadow-swiss-sm'
              : 'text-muted hover:text-foreground'
          ]"
        >
          Créer un profil
        </button>
      </div>

      <!-- Auth Form -->
      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div>
          <label class="block text-xs font-bold uppercase tracking-wider text-muted font-display mb-1.5">
            Pseudonyme Joueur FS25
          </label>
          <input
            v-model="username"
            type="text"
            required
            class="w-full bg-surface-raised border border-surface-border rounded-xl px-3.5 py-2.5 text-sm text-foreground focus:outline-none focus:border-primary font-sans"
            placeholder="ex: AgriPro_Alex"
          />
        </div>

        <div v-if="isRegister">
          <label class="block text-xs font-bold uppercase tracking-wider text-muted font-display mb-1.5">
            Adresse E-mail
          </label>
          <input
            v-model="email"
            type="email"
            required
            class="w-full bg-surface-raised border border-surface-border rounded-xl px-3.5 py-2.5 text-sm text-foreground focus:outline-none focus:border-primary font-sans"
            placeholder="alex@farmlink.io"
          />
        </div>

        <div>
          <label class="block text-xs font-bold uppercase tracking-wider text-muted font-display mb-1.5">
            Mot de passe / Token
          </label>
          <div class="relative">
            <input
              v-model="password"
              type="password"
              required
              class="w-full bg-surface-raised border border-surface-border rounded-xl px-3.5 py-2.5 text-sm text-foreground focus:outline-none focus:border-primary font-sans"
              placeholder="••••••••••••"
            />
            <Lock class="w-4 h-4 absolute right-3.5 top-3 text-muted pointer-events-none" />
          </div>
        </div>

        <!-- Role Selector -->
        <div>
          <label class="block text-xs font-bold uppercase tracking-wider text-muted font-display mb-2">
            Rôle sur la plateforme
          </label>
          <div class="grid grid-cols-2 gap-2.5">
            <button
              type="button"
              @click="selectedRole = 'player'"
              :class="[
                'p-3.5 border text-left rounded-xl transition-all',
                selectedRole === 'player'
                  ? 'bg-primary/10 border-primary text-foreground shadow-swiss-sm'
                  : 'bg-surface-raised border-surface-border text-muted hover:border-foreground/30',
              ]"
            >
              <div class="flex items-center justify-between mb-1.5">
                <UserCheck class="w-4 h-4 text-primary" />
                <CheckCircle2 v-if="selectedRole === 'player'" class="w-4 h-4 text-primary" />
              </div>
              <div class="text-xs font-bold uppercase tracking-wider font-display">Joueur Co-op</div>
              <div class="text-[10px] text-muted mt-0.5">Sync 1-clic & jeu</div>
            </button>

            <button
              type="button"
              @click="selectedRole = 'admin'"
              :class="[
                'p-3.5 border text-left rounded-xl transition-all',
                selectedRole === 'admin'
                  ? 'bg-brand-dark/10 dark:bg-purple-950/40 border-brand-dark dark:border-purple-600 text-foreground shadow-swiss-sm'
                  : 'bg-surface-raised border-surface-border text-muted hover:border-foreground/30',
              ]"
            >
              <div class="flex items-center justify-between mb-1.5">
                <ShieldCheck class="w-4 h-4 text-foreground" />
                <CheckCircle2 v-if="selectedRole === 'admin'" class="w-4 h-4 text-foreground" />
              </div>
              <div class="text-xs font-bold uppercase tracking-wider font-display">Admin Serveur</div>
              <div class="text-[10px] text-muted mt-0.5">Gestion du modpack</div>
            </button>
          </div>
        </div>

        <!-- Submit Button -->
        <Button
          type="submit"
          variant="primary"
          size="lg"
          class="w-full rounded-full shadow-swiss-md mt-4"
        >
          <span>{{ isRegister ? 'Créer le profil joueur' : 'Ouvrir la session' }}</span>
          <ArrowRight class="w-4 h-4" />
        </Button>
      </form>
    </div>

    <!-- Minimal Footer -->
    <div class="mt-8 text-center text-xs text-muted font-sans">
      © 2026 FarmLink
    </div>
  </div>
</template>
