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
  <div class="bg-surface rounded-2xl p-6 sm:p-8 border-2 border-dashed border-surface-border hover:border-primary transition-all shadow-swiss-sm">
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
      class="flex flex-col items-center justify-center text-center cursor-pointer py-6"
      @click="triggerFilePicker"
    >
      <div
        :class="[
          'w-14 h-14 rounded-2xl flex items-center justify-center mb-4 transition-transform border',
          isDragging ? 'scale-110 bg-primary/20 border-primary text-primary' : 'bg-surface-raised border-surface-border text-muted',
        ]"
      >
        <UploadCloud class="w-7 h-7" />
      </div>

      <h3 class="text-base font-bold uppercase tracking-wider text-foreground font-display">
        Glissez-déposez une archive mod <span class="text-primary font-mono lowercase">.zip</span>
      </h3>
      <p class="text-xs text-muted max-w-sm mt-1.5 font-sans leading-relaxed">
        Extraction automatique du fichier <code class="text-xs text-foreground bg-surface-raised px-1.5 py-0.5 rounded font-mono">modDesc.xml</code>, calcul des hachages SHA-256 et téléversement direct vers S3.
      </p>

      <div class="mt-5 flex items-center gap-2">
        <Button size="sm" variant="outline" class="rounded-full" @click.stop="triggerFilePicker">
          <FileArchive class="w-4 h-4 text-primary" />
          Parcourir les fichiers locaux
        </Button>
      </div>
    </div>

    <!-- Quick test presets -->
    <div class="border-t border-surface-border mt-4 pt-3 flex flex-wrap items-center justify-between gap-2 text-xs text-muted">
      <span class="flex items-center gap-1.5 font-mono text-[11px]">
        <Check class="w-3.5 h-3.5 text-primary" />
        Simulations rapides :
      </span>
      <div class="flex items-center gap-2">
        <button
          type="button"
          @click.stop="uploadDemoMod('FS25_Kuhn_Balandier_v1.zip', 45200000)"
          class="px-2.5 py-1 bg-surface-raised hover:bg-surface-hover border border-surface-border text-foreground text-xs rounded-full transition-colors font-mono"
        >
          + FS25_Kuhn_Balandier.zip
        </button>
        <button
          type="button"
          @click.stop="uploadDemoMod('FS25_corrupted_modDesc.zip', 1200000)"
          class="px-2.5 py-1 bg-surface-raised hover:border-red-500/50 text-red-500 border border-surface-border text-xs rounded-full transition-colors flex items-center gap-1 font-mono"
        >
          <AlertOctagon class="w-3 h-3" />
          Test Erreur XML
        </button>
      </div>
    </div>
  </div>
</template>
