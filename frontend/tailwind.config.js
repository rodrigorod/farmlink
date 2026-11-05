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
        background: '#121417',
        surface: '#1a1d20',
        'surface-raised': '#22272c',
        'surface-border': '#2d333b',
        'surface-hover': '#262c33',
        border: '#2d333b',
        input: '#22272c',
        ring: '#438E33',
        foreground: '#F3F4F6',
        muted: {
          DEFAULT: '#9CA3AF',
          foreground: '#6B7280',
        },
        primary: {
          DEFAULT: '#438E33',
          foreground: '#FFFFFF',
          hover: '#4ea83c',
          muted: '#2a5a21',
        },
        accent: {
          DEFAULT: '#E67E22',
          foreground: '#FFFFFF',
          hover: '#f39c12',
          muted: '#8e4b10',
        },
        destructive: {
          DEFAULT: '#DC2626',
          foreground: '#FFFFFF',
        },
        warning: {
          DEFAULT: '#E67E22',
          foreground: '#FFFFFF',
        },
        success: {
          DEFAULT: '#438E33',
          foreground: '#FFFFFF',
        },
      },
      fontFamily: {
        sans: ['Inter', 'Barlow', 'system-ui', '-apple-system', 'sans-serif'],
        mono: ['JetBrains Mono', 'Fira Code', 'ui-monospace', 'monospace'],
      },
      borderRadius: {
        lg: '6px',
        md: '4px',
        sm: '2px',
      },
      animation: {
        'industrial-blink': 'industrial-blink 1.2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'subtle-pulse': 'subtle-pulse 2s ease-in-out infinite',
      },
      keyframes: {
        'industrial-blink': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.2' },
        },
        'subtle-pulse': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.6' },
        },
      },
    },
  },
  plugins: [],
}
