import { createApp } from 'vue'
import App from './App.vue'
import './styles/main.css'
import { i18n } from './i18n'

const app = createApp(App)
app.use(i18n)

app.mount('#app')

setTimeout(() => {
  const loading = document.getElementById('loading-screen') as HTMLElement | null
  const appEl = document.getElementById('app') as HTMLElement | null

  if (loading) {
    loading.classList.add('fade-out')
    setTimeout(() => loading.remove(), 300)
  }

  if (appEl) {
    appEl.style.display = ''
  }
}, 100)