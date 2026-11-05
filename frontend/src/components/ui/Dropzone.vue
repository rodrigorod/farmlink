<script setup lang="ts">
import { ref } from 'vue'
import { UploadCloud, FileArchive, AlertOctagon, Check } from 'lucide-vue-next'
import Button from './Button.vue'

const emit = defineEmits<{
  (e: 'file-selected', file: File | { name: string; size: number }): void
}>()

const isDragging = ref(false)
const isUploading = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

function handleDragOver(e: DragEvent) {
  e.preventDefault()
  isDragging.value = true
}

function handleDragLeave(e: DragEvent) {
  e.preventDefault()
  isDragging.value = false
}

function handleDrop(e: DragEvent) {
  e.preventDefault()
  isDragging.value = false
  if (e.dataTransfer && e.dataTransfer.files.length > 0) {
    processFile(e.dataTransfer.files[0])
  }
}

function handleFileInput(e: Event) {
  const target = e.target as HTMLInputElement
  if (target.files && target.files.length > 0) {
    processFile(target.files[0])
  }
}

function triggerFilePicker() {
  fileInput.value?.click()
}

async function processFile(file: File | { name: string; size: number }) {
  isUploading.value = true
  emit('file-selected', file)
  setTimeout(() => {
    isUploading.value = false
  }, 400)
}

function uploadDemoMod(name: string, size: number) {
  processFile({ name, size } as File)
}
</script>

<template>
  <div class="industrial-panel p-6 border-dashed border-2 border-surface-border hover:border-primary transition-colors bg-surface/50">
    <input
      ref="fileInput"
      type="file"
      accept=".zip"
      class="hidden"
      @change="handleFileInput"
    />

    <div
      @dragover="handleDragOver"
      @dragleave="handleDragLeave"
      @drop="handleDrop"
      class="flex flex-col items-center justify-center text-center cursor-pointer py-8"
      @click="triggerFilePicker"
    >
      <div
        :class="[
          'w-16 h-16 rounded-sm flex items-center justify-center mb-4 transition-transform border',
          isDragging ? 'scale-110 bg-primary/20 border-primary text-primary' : 'bg-surface-raised border-surface-border text-muted',
        ]"
      >
        <UploadCloud class="w-8 h-8" />
      </div>

      <h3 class="text-base font-semibold text-foreground tracking-wide">
        Glissez-déposez une archive de mod <span class="text-primary font-mono">.zip</span>
      </h3>
      <p class="text-xs text-muted max-w-sm mt-1">
        Le backend extrait automatiquement le <code class="text-xs text-foreground bg-surface-raised px-1 py-0.5 rounded">modDesc.xml</code>, calcule les empreintes MD5/SHA-256 et téléverse vers le stockage S3.
      </p>

      <div class="mt-4 flex items-center gap-2">
        <Button size="sm" variant="outline" @click.stop="triggerFilePicker">
          <FileArchive class="w-4 h-4 text-primary" />
          Parcourir les fichiers
        </Button>
      </div>
    </div>

    <!-- Quick test presets -->
    <div class="border-t border-surface-border mt-4 pt-3 flex flex-wrap items-center justify-between gap-2 text-xs text-muted">
      <span class="flex items-center gap-1.5 font-mono text-[11px]">
        <Check class="w-3.5 h-3.5 text-primary" />
        Simulations rapides de test :
      </span>
      <div class="flex items-center gap-2">
        <button
          type="button"
          @click.stop="uploadDemoMod('FS25_Kuhn_Balandier_v1.zip', 45200000)"
          class="px-2 py-1 bg-surface-raised hover:bg-surface-hover border border-surface-border text-foreground text-xs rounded-sm transition-colors"
        >
          + FS25_Kuhn_Balandier.zip
        </button>
        <button
          type="button"
          @click.stop="uploadDemoMod('FS25_corrupted_modDesc.zip', 1200000)"
          class="px-2 py-1 bg-surface-raised hover:border-red-600/50 text-red-400 border border-surface-border text-xs rounded-sm transition-colors flex items-center gap-1"
        >
          <AlertOctagon class="w-3 h-3" />
          Test Erreur XML
        </button>
      </div>
    </div>
  </div>
</template>
