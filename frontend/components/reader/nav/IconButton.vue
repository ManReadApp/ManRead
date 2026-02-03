<script setup lang="ts">
defineProps({
  label: {
    type: String,
    required: true
  }
})
</script>

<template>
  <div class="m-1 px-1 py-0.5 cursor-pointer rounded hover:bg-white/10 flex flex-col">
    <div class="flex-1 overflow-hidden justify-center flex items-center p-0.5">
      <slot/>
    </div>
    <p class="text-white text-xs select-none">{{label}}</p>
  </div>
</template>

<style scoped>

</style>