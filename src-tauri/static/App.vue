<template>
  
  <div class="flex flex-col min-h-screen">
    <NavBar 
      :is-menu-open="isMenuOpen"
      @toggle-menu="isMenuOpen = !isMenuOpen"
      @close-menu="isMenuOpen = false"
      @open-about="openAboutModal"
    />
    
    <main class="flex-1 bg-slate-100">
      <FileBrowser />
    </main>
    
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

<script setup>
import { ref } from 'vue'
import NavBar from './components/web/NavBar.vue'
import FileBrowser from './components/web/FileBrowser.vue'
import AboutModal from './components/web/AboutModal.vue'

const isMenuOpen = ref(false)
const aboutOpen = ref(false)

function openAboutModal() {
  isMenuOpen.value = false
  aboutOpen.value = true
}
</script>