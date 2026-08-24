<template>
  <div class="flex flex-col min-h-screen">
    <NavBar
      :is-menu-open="isMenuOpen"
      @toggle-menu="isMenuOpen = !isMenuOpen"
      @close-menu="isMenuOpen = false"
      @open-about="openAboutModal"
    />

    <router-view />

    <footer class="w-full bg-blue-700 text-white py-4">
      <p class="text-center text-sm">
        © {{ new Date().getFullYear() }} OpenVue ·
        <a href="https://github.com/yunend/openvue" target="_blank" class="underline hover:text-blue-200">GitHub</a>
      </p>
    </footer>

    <AboutModal
      v-if="aboutOpen"
      @close="aboutOpen = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import NavBar from './components/web/NavBar.vue'
import AboutModal from './components/web/AboutModal.vue'

const isMenuOpen = ref(false)
const aboutOpen = ref(false)

function openAboutModal(): void {
  isMenuOpen.value = false
  aboutOpen.value = true
}
</script>