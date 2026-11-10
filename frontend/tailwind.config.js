/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ['class'],
  content: [
    './index.html',
    './src/**/*.{vue,js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      colors: {
        background: 'var(--color-bg)',
        surface: 'var(--color-surface)',
        'surface-raised': 'var(--color-surface-raised)',
        'surface-border': 'var(--color-surface-border)',
        'surface-hover': 'var(--color-surface-hover)',
        border: 'var(--color-surface-border)',
        input: 'var(--color-input)',
        ring: '#159D3B',
        foreground: 'var(--color-foreground)',
        muted: {
          DEFAULT: 'var(--color-muted)',
          foreground: 'var(--color-muted-fg)',
        },
        primary: {
          DEFAULT: '#159D3B',
          foreground: '#FFFFFF',
          hover: '#128A34',
          muted: 'var(--color-primary-muted)',
          dark: '#0B5620',
        },
        secondary: {
          DEFAULT: 'var(--color-surface-raised)',
          foreground: 'var(--color-foreground)',
          hover: 'var(--color-surface-hover)',
        },
        accent: {
          DEFAULT: '#159D3B',
          foreground: '#FFFFFF',
          hover: '#128A34',
        },
        destructive: {
          DEFAULT: '#DC2626',
          foreground: '#FFFFFF',
        },
        warning: {
          DEFAULT: '#D97706',
          foreground: '#FFFFFF',
        },
        success: {
          DEFAULT: '#159D3B',
          foreground: '#FFFFFF',
        },
        'brand-dark': '#0A0E0B',
        'brand-sand': '#EAE5DC',
      },
      fontFamily: {
        sans: ['"Helvetica Neue"', 'Helvetica', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'sans-serif'],
        display: ['Archivo', '"Helvetica Neue"', 'Helvetica', 'sans-serif'],
        mono: ['"JetBrains Mono"', 'Fira Code', 'ui-monospace', 'monospace'],
      },
      borderRadius: {
        none: '0px',
        sm: '2px',
        md: '4px',
        DEFAULT: '6px',
        lg: '8px',
        xl: '12px',
        '2xl': '16px',
        '3xl': '24px',
        full: '9999px',
      },
      boxShadow: {
        'swiss-sm': '0 1px 2px 0 rgba(0, 0, 0, 0.04)',
        'swiss-md': '0 4px 12px 0 rgba(0, 0, 0, 0.05)',
        'swiss-lg': '0 10px 25px -3px rgba(0, 0, 0, 0.06)',
      },
      letterSpacing: {
        tighter: '-0.04em',
        tight: '-0.02em',
        normal: '0',
        wide: '0.02em',
        wider: '0.05em',
        widest: '0.12em',
      },
    },
  },
  plugins: [],
}
