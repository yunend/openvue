import { ref, watch, onMounted } from 'vue'

export type ThemeName = 'light' | 'dark' | 'sunset' | 'forest' | 'ocean'

export interface ThemeOption {
  name: ThemeName
  label: string
  icon: string
  class: string
}

export const THEMES: ThemeOption[] = [
  { name: 'light', label: '白天', icon: '☀️', class: '' },
  { name: 'dark', label: '黑夜', icon: '🌙', class: 'theme-dark' },
  { name: 'sunset', label: '日落', icon: '🌅', class: 'theme-sunset' },
  { name: 'forest', label: '森林', icon: '🌲', class: 'theme-forest' },
  { name: 'ocean', label: '海洋', icon: '🌊', class: 'theme-ocean' },
]

export function useTheme() {
  const currentTheme = ref<ThemeName>('light')

  function setTheme(theme: ThemeName): void {
    currentTheme.value = theme
    const themeOption = THEMES.find(t => t.name === theme)
    if (themeOption) {
      const body = document.body
      const html = document.documentElement
      // 移除所有主题类
      body.classList.remove('theme-dark', 'theme-sunset', 'theme-forest', 'theme-ocean')
      // 添加新主题类
      if (themeOption.class) {
        body.classList.add(themeOption.class)
      }
      // 黑夜模式下同时添加 Tailwind dark 类
      if (theme === 'dark') {
        html.classList.add('dark')
      } else {
        html.classList.remove('dark')
      }
      // 保存到 localStorage
      localStorage.setItem('app-theme', theme)
    }
  }

  function getCurrentThemeOption(): ThemeOption {
    return THEMES.find(t => t.name === currentTheme.value) || THEMES[0]
  }

  function nextTheme(): void {
    const currentIndex = THEMES.findIndex(t => t.name === currentTheme.value)
    const nextIndex = (currentIndex + 1) % THEMES.length
    setTheme(THEMES[nextIndex].name)
  }

  onMounted(() => {
    const savedTheme = localStorage.getItem('app-theme') as ThemeName | null
    if (savedTheme && THEMES.find(t => t.name === savedTheme)) {
      setTheme(savedTheme)
    }
  })

  watch(currentTheme, (newTheme) => {
    setTheme(newTheme)
  })

  return {
    currentTheme,
    themes: THEMES,
    setTheme,
    getCurrentThemeOption,
    nextTheme,
  }
}