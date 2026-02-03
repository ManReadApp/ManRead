<script setup lang="ts">
defineProps({
  images: {
    type: Array,
    required: true
  },
  leftToRight: {
    type: Boolean,
    required: true
  }
})
</script>

<template>
  <div class="w-full h-full relative">
    <img class="w-full absolute top-0" :style="`${leftToRight ? 'left' : 'right'}: ${item.offset}px`" alt="manga image"
         v-for="item in images" />
  </div>
</template>

<style scoped>

</style>