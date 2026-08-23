// src-tauri/static/router/index.js
import { createRouter, createWebHashHistory } from 'vue-router'
import MainView from '../views/MainView.vue'
import UploadView from '../views/UploadView.vue'

const routes = [
  { path: '/', name: 'home', component: MainView },
  { path: '/upload', name: 'upload', component: UploadView },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router