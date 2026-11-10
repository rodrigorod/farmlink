<script setup lang="ts">
import { ref } from 'vue'
import { useServersStore } from '@/stores/servers'
import {
  X,
  Server as ServerIcon,
  Plus,
  KeyRound,
  Compass,
  Radio,
  Hash,
} from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'

defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'created', serverId: string): void
  (e: 'joined', serverId: string): void
}>()

const serversStore = useServersStore()

const activeTab = ref<'create' | 'join'>('create')
const isSubmitting = ref(false)

// Form fields for server creation
const createForm = ref({
  name: '',
  mapName: 'Zielonka (Pologne)',
  ipAddress: '',
  port: 10823,
})

// Form fields for joining server
const joinForm = ref({
  inviteCode: '',
})

async function handleCreateServer() {
  if (!createForm.value.name.trim()) return

  isSubmitting.value = true
  try {
    const newServer = await serversStore.createServer({
      name: createForm.value.name,
      mapName: createForm.value.mapName,
      ipAddress: createForm.value.ipAddress,
      port: Number(createForm.value.port) || 10823,
    })

    if (newServer) {
      createForm.value.name = ''
      createForm.value.ipAddress = ''
      createForm.value.port = 10823
      emit('created', newServer.id)
      emit('close')
    }
  } finally {
    isSubmitting.value = false
  }
}

async function handleJoinServer() {
  if (!joinForm.value.inviteCode.trim()) return

  isSubmitting.value = true
  try {
    const joined = await serversStore.joinServerByCode(joinForm.value.inviteCode)
    if (joined) {
      joinForm.value.inviteCode = ''
      emit('joined', joined.id)
      emit('close')
    }
  } finally {
    isSubmitting.value = false
  }
}

function handleClose() {
  if (!isSubmitting.value) {
    emit('close')
  }
}
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm"
    @click.self="handleClose"
    @keydown.esc="handleClose"
  >
    <div
      class="bg-surface w-full max-w-lg shadow-swiss-lg border border-surface-border text-foreground rounded-3xl overflow-hidden"
      role="dialog"
      aria-modal="true"
    >
      <!-- Modal Header -->
      <div class="p-6 border-b border-surface-border flex items-center justify-between bg-surface-raised">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-full bg-primary/10 border border-primary/30 flex items-center justify-center text-primary">
            <ServerIcon class="w-4 h-4" />
          </div>
          <div>
            <h2 class="text-base font-black uppercase tracking-tight text-foreground font-display">
              Gestion de Serveur FS25
            </h2>
            <p class="text-xs font-mono text-muted">
              Déploiement hôte ou raccordement par code d'invitation
            </p>
          </div>
        </div>
        <button
          type="button"
          class="w-8 h-8 rounded-full bg-surface border border-surface-border flex items-center justify-center text-muted hover:text-foreground transition-colors"
          @click="handleClose"
          title="Fermer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Tab Switcher (Segmented Pill Style) -->
      <div class="p-4 bg-surface border-b border-surface-border">
        <div class="grid grid-cols-2 p-1 bg-surface-raised border border-surface-border rounded-xl">
          <button
            type="button"
            class="py-2 text-xs font-bold uppercase tracking-wider rounded-lg transition-all flex items-center justify-center gap-2 font-display"
            :class="[
              activeTab === 'create'
                ? 'bg-surface text-foreground shadow-swiss-sm'
                : 'text-muted hover:text-foreground'
            ]"
            @click="activeTab = 'create'"
          >
            <Plus class="w-3.5 h-3.5 text-primary" />
            Créer un Serveur (Hôte)
          </button>

          <button
            type="button"
            class="py-2 text-xs font-bold uppercase tracking-wider rounded-lg transition-all flex items-center justify-center gap-2 font-display"
            :class="[
              activeTab === 'join'
                ? 'bg-surface text-foreground shadow-swiss-sm'
                : 'text-muted hover:text-foreground'
            ]"
            @click="activeTab = 'join'"
          >
            <KeyRound class="w-3.5 h-3.5 text-primary" />
            Rejoindre (Code)
          </button>
        </div>
      </div>

      <!-- Tab 1: Create Server (Owner) -->
      <form
        v-if="activeTab === 'create'"
        @submit.prevent="handleCreateServer"
        class="p-6 space-y-4 text-xs font-sans"
      >
        <div>
          <label class="block text-xs font-bold uppercase tracking-wider text-muted font-display mb-1.5">
            Nom de la Ferme / Serveur <span class="text-primary">*</span>
          </label>
          <input
            v-model="createForm.name"
            type="text"
            required
            maxlength="100"
            placeholder="Ex : GAEC des Plaines d'Alsace"
            class="w-full bg-surface-raised border border-surface-border rounded-xl px-3.5 py-2.5 text-foreground focus:outline-none focus:border-primary text-xs placeholder:text-muted/60"
          />
        </div>

        <div>
          <label class="block text-xs font-bold uppercase tracking-wider text-muted font-display mb-1.5 flex items-center gap-1.5">
            <Compass class="w-3.5 h-3.5 text-primary" />
            Carte Agricole FS25
          </label>
          <select
            v-model="createForm.mapName"
            class="w-full bg-surface-raised border border-surface-border rounded-xl px-3.5 py-2.5 text-foreground focus:outline-none focus:border-primary text-xs cursor-pointer font-sans"
          >
            <option value="Zielonka (Pologne)">Zielonka (Pologne)</option>
            <option value="Riverbend Springs (USA)">Riverbend Springs (USA)</option>
            <option value="Haut-Beyleron (France)">Haut-Beyleron (France)</option>
            <option value="Erlengrat (Suisse)">Erlengrat (Suisse)</option>
            <option value="Ferme Personnalisée (Mod Map)">Ferme Personnalisée (Mod Map)</option>
          </select>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <div class="sm:col-span-2">
            <label class="block text-xs font-bold uppercase tracking-wider text-muted font-display mb-1.5 flex items-center gap-1.5">
              <Radio class="w-3.5 h-3.5 text-primary" />
              Adresse IP ou Hôte
            </label>
            <input
              v-model="createForm.ipAddress"
              type="text"
              placeholder="Ex: 192.168.1.50 ou srv.domaine.fr"
              class="w-full bg-surface-raised border border-surface-border rounded-xl px-3.5 py-2.5 text-foreground focus:outline-none focus:border-primary text-xs font-mono placeholder:text-muted/60"
            />
          </div>

          <div>
            <label class="block text-xs font-bold uppercase tracking-wider text-muted font-display mb-1.5 flex items-center gap-1.5">
              <Hash class="w-3.5 h-3.5 text-primary" />
              Port FS25
            </label>
            <input
              v-model.number="createForm.port"
              type="number"
              min="1"
              max="65535"
              placeholder="10823"
              class="w-full bg-surface-raised border border-surface-border rounded-xl px-3.5 py-2.5 text-foreground focus:outline-none focus:border-primary text-xs font-mono"
            />
          </div>
        </div>

        <div class="p-3.5 bg-surface-raised border border-surface-border rounded-xl text-muted text-[11px] leading-relaxed">
          <strong class="text-foreground">Information Propriétaire :</strong> Vous aurez les droits complets pour uploader des archives de mods, activer/désactiver des versions et partager le code d'accès généré.
        </div>

        <div class="pt-4 flex items-center justify-end gap-2 border-t border-surface-border">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            @click="handleClose"
            :disabled="isSubmitting"
            class="rounded-full"
          >
            Annuler
          </Button>
          <Button
            type="submit"
            variant="primary"
            size="sm"
            :disabled="isSubmitting || !createForm.name.trim()"
            class="rounded-full"
          >
            <Plus class="w-4 h-4" />
            {{ isSubmitting ? 'Création en cours...' : 'Initialiser le Serveur' }}
          </Button>
        </div>
      </form>

      <!-- Tab 2: Join Server by Code -->
      <form
        v-else
        @submit.prevent="handleJoinServer"
        class="p-6 space-y-4 text-xs font-sans"
      >
        <div>
          <label class="block text-xs font-bold uppercase tracking-wider text-muted font-display mb-1.5">
            Code d'invitation de la ferme <span class="text-primary">*</span>
          </label>
          <div class="relative">
            <input
              v-model="joinForm.inviteCode"
              type="text"
              required
              maxlength="20"
              placeholder="Ex : FLK-7KM89X ou ZIELONKA2025"
              class="w-full bg-surface-raised border border-surface-border rounded-xl px-3.5 py-3 text-foreground uppercase tracking-widest font-black focus:outline-none focus:border-primary text-sm font-mono placeholder:text-muted/40 placeholder:tracking-normal placeholder:font-normal"
            />
            <KeyRound class="w-4 h-4 absolute right-3.5 top-3.5 text-muted pointer-events-none" />
          </div>
          <p class="text-[11px] text-muted mt-1.5 font-sans">
            Demandez le code d'invitation à l'hôte de la partie pour synchroniser automatiquement les mods du serveur.
          </p>
        </div>

        <div class="p-3.5 bg-surface-raised border border-surface-border rounded-xl text-muted text-[11px] leading-relaxed">
          <strong class="text-foreground">Synchronisation automatique :</strong> Dès la validation du code, le serveur apparaîtra sur votre tableau de bord et comparera vos fichiers locaux avec le manifeste officiel.
        </div>

        <div class="pt-4 flex items-center justify-end gap-2 border-t border-surface-border">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            @click="handleClose"
            :disabled="isSubmitting"
            class="rounded-full"
          >
            Annuler
          </Button>
          <Button
            type="submit"
            variant="primary"
            size="sm"
            :disabled="isSubmitting || !joinForm.inviteCode.trim()"
            class="rounded-full"
          >
            <KeyRound class="w-4 h-4" />
            {{ isSubmitting ? 'Raccordement...' : 'Rejoindre la Ferme' }}
          </Button>
        </div>
      </form>
    </div>
  </div>
</template>
