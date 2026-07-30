<template>
  <nav class="w-full bg-blue-700">
    <section class="h-20 w-10/12 mx-auto font-mono text-white flex items-center gap-x-3">
      <a class="text-4xl px-5 py-3 hover:translate-x-5 duration-300" href="/">OpenVue</a>
      
      <ul class="hidden md:flex gap-x-5 mx-3 items-center">
        <li class="p-3 hover:bg-blue-900/50 cursor-pointer"><a href="/">主页</a></li>
        <li class="p-3 hover:bg-blue-900/50 cursor-pointer"><a href="/upload.html">上传文件</a></li>
      </ul>
      
      <ul class="hidden lg:flex mx-3 items-center ml-auto">
        <li 
          class="p-3 hover:bg-blue-900/50 cursor-pointer"
          @click="$emit('open-about')"
        >
          <a href="#about">关于</a>
        </li>
      </ul>
      
      <button 
        @click="$emit('toggle-menu')"
        class="ml-auto md:hidden px-5 py-3 bg-blue-900/30 hover:bg-blue-900/70 duration-300"
      >
        <svg viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg" width="35" height="35">
          <path d="M0.001024 256c0-28.2624 23.296-51.2 50.7904-51.2h922.4192c28.0576 0 50.7904 22.7328 50.7904 51.2 0 28.2624-23.296 51.2-50.7904 51.2H50.791424A50.8416 50.8416 0 0 1 0.001024 256z m0 256c0-28.2624 23.296-51.2 50.7904-51.2h922.4192c28.0576 0 50.7904 22.7328 50.7904 51.2 0 28.2624-23.296 51.2-50.7904 51.2H50.791424A50.8416 50.8416 0 0 1 0.001024 512z m0 256c0-28.2624 23.296-51.2 50.7904-51.2h922.4192c28.0576 0 50.7904 22.7328 50.7904 51.2 0 28.2624-23.296 51.2-50.7904 51.2H50.791424A50.8416 50.8416 0 0 1 0.001024 768z" fill="white"></path>
        </svg>
      </button>
    </section>
    
    <div 
      id="mobile-menu" 
      class="h-screen w-full bg-blue-700 fixed top-0 md:hidden duration-300"
      :class="isMenuOpen ? 'left-0' : '-left-full'"
    >
      <ul class="h-full my-5 w-10/12 mx-auto text-white flex-col">
        <li class="p-3 text-right">
          <button 
            @click="$emit('close-menu')"
            class="px-5 py-3 hover:bg-blue-900"
          >
            <svg viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg" width="20" height="20">
              <path d="M981.548 981.476c15.722-15.722 15.722-41.143 0-56.696L92.103 35.335c-15.722-15.722-41.143-15.722-56.696 0-15.722 15.722-15.722 41.143 0 56.696l889.278 889.278c15.722 15.89 41.143 15.89 56.867 0.167z m7.192-938.952c-15.722-15.722-41.143-15.722-56.696 0L42.599 931.969c-15.722 15.722-15.722 41.143 0 56.696 15.722 15.722 41.143 15.722 56.696 0L988.573 99.387c15.722-15.722 15.722-41.143 0.167-56.867z" fill="white"></path>
            </svg>
          </button>
        </li>
        <li class="p-3 hover:bg-blue-900/50 cursor-pointer duration-300"><a href="/">主页</a></li>
        <li class="p-3 hover:bg-blue-900/50 cursor-pointer duration-300"><a href="/upload.html">上传文件</a></li>
        <li 
          class="p-3 hover:bg-blue-900/50 cursor-pointer duration-300"
          @click="$emit('open-about'); $emit('close-menu')"
        >
          <a href="#about">关于</a>
        </li>
      </ul>
    </div>
  </nav>
</template>

<script setup>
defineProps({
  isMenuOpen: Boolean
})

defineEmits(['toggle-menu', 'close-menu', 'open-about'])
</script>