import { createI18n } from 'vue-i18n'
import en from './locales/en'
import zh from './locales/zh'

export type Locale = 'zh' | 'en'

const STORAGE_KEY = 'app-locale'

function detectLocale(): Locale {
  try {
    const saved = localStorage.getItem(STORAGE_KEY) as Locale | null
    if (saved === 'zh' || saved === 'en') {
      return saved
    }
  } catch {
    /* ignore */
  }
  const nav = typeof navigator !== 'undefined' ? navigator.language || '' : ''
  return nav.toLowerCase().startsWith('zh') ? 'zh' : 'en'
}

export const i18n = createI18n({
  legacy: false,
  locale: detectLocale(),
  fallbackLocale: 'en',
  messages: { zh, en }
})

export function setLocale(locale: Locale) {
  i18n.global.locale.value = locale
  try {
    localStorage.setItem(STORAGE_KEY, locale)
    document.documentElement.setAttribute('lang', locale)
  } catch {
    /* ignore */
  }
}

export function getLocale(): Locale {
  return (i18n.global.locale.value as Locale) || 'en'
}