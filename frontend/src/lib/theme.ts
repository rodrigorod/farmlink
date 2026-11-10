import { ref } from 'vue'

export type Theme = 'light' | 'dark'

function getInitialTheme(): Theme {
  if (typeof window === 'undefined') return 'light'
  const saved = localStorage.getItem('farmlink_theme') as Theme | null
  if (saved === 'dark' || saved === 'light') {
    return saved
  }
  // Default to light to match the pristine Swiss editorial canvas references
  return 'light'
}

export const currentTheme = ref<Theme>(getInitialTheme())

export function applyTheme(theme: Theme) {
  currentTheme.value = theme
  if (typeof document !== 'undefined') {
    const root = document.documentElement
    if (theme === 'dark') {
      root.classList.add('dark')
    } else {
      root.classList.remove('dark')
    }
  }
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('farmlink_theme', theme)
  }
}

export function toggleTheme() {
  const next = currentTheme.value === 'dark' ? 'light' : 'dark'
  applyTheme(next)
}

// Initial apply
if (typeof window !== 'undefined') {
  applyTheme(currentTheme.value)
}
