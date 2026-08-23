<template>
  <div class="toast-container" :class="{ show: toast.show }">
    <div class="toast" :class="toast.type" v-if="toast.show">
      {{ toast.message }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { useToast } from '../../composables/useToast'

const { toast } = useToast()
</script>