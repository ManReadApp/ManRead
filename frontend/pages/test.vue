<script setup lang="ts">
const {$manRead} = useNuxtApp()
const r = ref(null)
onMounted(async () => {
  console.log(`Bearer ${await getAccessToken()}`)
  const data = await $manRead('/api/v1/external/statistics', {
    method: "POST",
    headers: {Authorization: `Bearer ${await getAccessToken()}`},
  })
  r.value = data
});

</script>

<template>
  <AuthGuard>
    <pre class="h-full overflow-auto">
      {{ JSON.stringify(r,null, 2) }}
    </pre>
  </AuthGuard>
</template>

<style scoped>

</style>