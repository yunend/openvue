/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: [
    './src/**/*.{vue,js,ts,jsx,tsx}',
    './src-tauri/base/**/*.{vue,js,ts,jsx,tsx,html}'
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#e8eaf6',
          100: '#c5cae9',
          200: '#9fa8da',
          300: '#7986cb',
          400: '#5c6bc0',
          500: '#3f51b5',
          600: '#3949ab',
          700: '#303f9f',
          800: '#283593',
          900: '#1a237e'
        },
        'nav-bg': 'var(--nav-bg)',
        'nav-text': 'var(--nav-text)',
        'page-bg': 'var(--page-bg)',
        'card-bg': 'var(--card-bg)',
        'text-primary': 'var(--text-primary)',
        'text-secondary': 'var(--text-secondary)',
        'footer-bg': 'var(--footer-bg)',
        'footer-text': 'var(--footer-text)',
        'border-color': 'var(--border-color)',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0', transform: 'translateY(6px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' }
        },
        slideDown: {
          '0%': { opacity: '0', transform: 'translateY(-100%)' },
          '100%': { opacity: '1', transform: 'translateY(0)' }
        },
        pulseBtn: {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.6' }
        }
      },
      animation: {
        fadeIn: 'fadeIn 0.22s ease',
        slideDown: 'slideDown 0.4s ease',
        pulseBtn: 'pulseBtn 1.5s ease-in-out infinite'
      }
    },
  },
  plugins: [],
}