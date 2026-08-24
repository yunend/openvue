// src-tauri/static/router/index.ts
import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'
import MainView from '../views/MainView.vue'
import UploadView from '../views/UploadView.vue'

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'home', component: MainView },
  { path: '/upload', name: 'upload', component: UploadView },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router