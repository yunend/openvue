import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN'
import en from './locales/en'

export type LocaleKey = 'zh-CN' | 'en'

const i18n = createI18n({
  legacy: false,
  locale: localStorage.getItem('app-locale') || navigator.language || 'zh-CN',
  fallbackLocale: 'en',
  messages: {
    'zh-CN': zhCN,
    'en': en,
  },
})

export function setLocale(locale: LocaleKey): void {
  i18n.global.locale.value = locale
  localStorage.setItem('app-locale', locale)
  document.documentElement.lang = locale
}

export function getCurrentLocale(): LocaleKey {
  return (i18n.global.locale.value as LocaleKey) || 'zh-CN'
}

export default i18n