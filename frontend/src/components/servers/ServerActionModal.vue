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
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-xs"
    @click.self="handleClose"
    @keydown.esc="handleClose"
  >
    <div
      class="industrial-panel bg-surface w-full max-w-lg shadow-2xl border border-surface-border text-foreground overflow-hidden"
      role="dialog"
      aria-modal="true"
    >
      <!-- Modal Header -->
      <div class="p-5 border-b border-surface-border flex items-center justify-between bg-surface-raised">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-sm bg-primary/20 border border-primary/40 flex items-center justify-center text-primary">
            <ServerIcon class="w-4 h-4" />
          </div>
          <div>
            <h2 class="text-sm font-black uppercase tracking-wider text-foreground">
              Gestion de Serveur FS25
            </h2>
            <p class="text-[11px] font-mono text-muted">
              Déploiement hôte ou raccordement par code d'invitation
            </p>
          </div>
        </div>
        <button
          type="button"
          class="p-1 text-muted hover:text-foreground transition-colors"
          @click="handleClose"
          title="Fermer"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Tab Switcher -->
      <div class="grid grid-cols-2 border-b border-surface-border bg-surface text-xs font-mono uppercase tracking-wider">
        <button
          type="button"
          class="py-3 px-4 text-center border-b-2 font-bold transition-all flex items-center justify-center gap-2"
          :class="[
            activeTab === 'create'
              ? 'border-primary text-primary bg-primary/5'
              : 'border-transparent text-muted hover:text-foreground hover:bg-surface-raised'
          ]"
          @click="activeTab = 'create'"
        >
          <Plus class="w-3.5 h-3.5" />
          Créer (Hôte)
        </button>

        <button
          type="button"
          class="py-3 px-4 text-center border-b-2 font-bold transition-all flex items-center justify-center gap-2"
          :class="[
            activeTab === 'join'
              ? 'border-primary text-primary bg-primary/5'
              : 'border-transparent text-muted hover:text-foreground hover:bg-surface-raised'
          ]"
          @click="activeTab = 'join'"
        >
          <KeyRound class="w-3.5 h-3.5" />
          Rejoindre (Code)
        </button>
      </div>

      <!-- Tab 1: Create Server (Owner) -->
      <form
        v-if="activeTab === 'create'"
        @submit.prevent="handleCreateServer"
        class="p-5 space-y-4 text-xs font-mono"
      >
        <div>
          <label class="block text-muted uppercase text-[10px] font-bold mb-1.5">
            Nom de la Ferme / Serveur <span class="text-primary">*</span>
          </label>
          <input
            v-model="createForm.name"
            type="text"
            required
            maxlength="100"
            placeholder="Ex : GAEC des Plaines d'Alsace"
            class="w-full bg-surface-raised border border-surface-border rounded-sm px-3 py-2 text-foreground focus:outline-none focus:border-primary text-xs placeholder:text-muted/50"
          />
        </div>

        <div>
          <label class="block text-muted uppercase text-[10px] font-bold mb-1.5 flex items-center gap-1.5">
            <Compass class="w-3 h-3 text-primary" />
            Carte Agricole FS25
          </label>
          <select
            v-model="createForm.mapName"
            class="w-full bg-surface-raised border border-surface-border rounded-sm px-3 py-2 text-foreground focus:outline-none focus:border-primary text-xs cursor-pointer"
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
            <label class="block text-muted uppercase text-[10px] font-bold mb-1.5 flex items-center gap-1.5">
              <Radio class="w-3 h-3 text-primary" />
              Adresse IP ou Hôte
            </label>
            <input
              v-model="createForm.ipAddress"
              type="text"
              placeholder="Ex: 192.168.1.50 ou srv.domaine.fr"
              class="w-full bg-surface-raised border border-surface-border rounded-sm px-3 py-2 text-foreground focus:outline-none focus:border-primary text-xs placeholder:text-muted/50"
            />
          </div>

          <div>
            <label class="block text-muted uppercase text-[10px] font-bold mb-1.5 flex items-center gap-1.5">
              <Hash class="w-3 h-3 text-primary" />
              Port FS25
            </label>
            <input
              v-model.number="createForm.port"
              type="number"
              min="1"
              max="65535"
              placeholder="10823"
              class="w-full bg-surface-raised border border-surface-border rounded-sm px-3 py-2 text-foreground focus:outline-none focus:border-primary text-xs"
            />
          </div>
        </div>

        <div class="p-3 bg-surface-raised border border-surface-border text-muted text-[11px] leading-relaxed">
          <strong class="text-foreground">Information Propriétaire :</strong> Vous aurez les droits complets pour uploader des archives de mods, activer/désactiver des versions et partager le code d'accès généré.
        </div>

        <div class="pt-2 flex items-center justify-end gap-2 border-t border-surface-border">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            @click="handleClose"
            :disabled="isSubmitting"
          >
            Annuler
          </Button>
          <Button
            type="submit"
            variant="primary"
            size="sm"
            :disabled="isSubmitting || !createForm.name.trim()"
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
        class="p-5 space-y-4 text-xs font-mono"
      >
        <div>
          <label class="block text-muted uppercase text-[10px] font-bold mb-1.5">
            Code d'invitation de la ferme <span class="text-primary">*</span>
          </label>
          <div class="relative">
            <input
              v-model="joinForm.inviteCode"
              type="text"
              required
              maxlength="20"
              placeholder="Ex : FLK-7KM89X ou ZIELONKA2025"
              class="w-full bg-surface-raised border border-surface-border rounded-sm px-3 py-2.5 text-foreground uppercase tracking-widest font-black focus:outline-none focus:border-primary text-sm placeholder:text-muted/40 placeholder:tracking-normal placeholder:font-normal"
            />
            <KeyRound class="w-4 h-4 absolute right-3 top-3 text-muted pointer-events-none" />
          </div>
          <p class="text-[11px] text-muted mt-1.5">
            Demandez le code d'invitation à l'hôte de la partie pour synchroniser automatiquement les mods du serveur.
          </p>
        </div>

        <div class="p-3 bg-surface-raised border border-surface-border text-muted text-[11px] leading-relaxed">
          <strong class="text-foreground">Synchronisation automatique :</strong> Dès la validation du code, le serveur apparaîtra sur votre tableau de bord et comparera vos fichiers locaux avec le manifeste officiel.
        </div>

        <div class="pt-2 flex items-center justify-end gap-2 border-t border-surface-border">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            @click="handleClose"
            :disabled="isSubmitting"
          >
            Annuler
          </Button>
          <Button
            type="submit"
            variant="primary"
            size="sm"
            :disabled="isSubmitting || !joinForm.inviteCode.trim()"
          >
            <KeyRound class="w-4 h-4" />
            {{ isSubmitting ? 'Raccordement...' : 'Rejoindre la Ferme' }}
          </Button>
        </div>
      </form>
    </div>
  </div>
</template>
