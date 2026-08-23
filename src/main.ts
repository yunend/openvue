import { createApp } from 'vue'
import App from './App.vue'
import './styles/main.css'

const app = createApp(App)

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