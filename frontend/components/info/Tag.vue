<script setup lang="ts">
defineProps<{ href: string, label: string }>()
</script>

<template>
  <a class="big-tag" :href="href">
    <span>{{ label }}</span>
  </a>
</template>

<style scoped>
.big-tag {
  align-items: center;
  border-radius: .25rem;
  display: inline-flex;
  font-size: .75rem;
  min-height: 1.75rem;
  padding: .3125rem .5rem;
  transition: all .1s ease-out;
  background: rgb(240, 241, 242);
}

.big-tag:hover {
  background: rgb(var(--main-color));
  color: white;
}
</style>