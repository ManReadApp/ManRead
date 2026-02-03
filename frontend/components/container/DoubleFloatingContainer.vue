<template>
  <div class="min-h-screen bg-gray-100 flex items-center justify-center p-4">
    <div
        class="grid grid-cols-2 lg:grid-cols-12 gap-8 w-full max-w-[1400px] items-start"
    >
      <!-- Empty spacer on large screens -->
      <div class="hidden lg:block lg:col-span-2"></div>

      <!-- Main Content (50%) -->
      <div
          class="col-span-1 lg:col-span-6 h-[80vh] overflow-auto bg-white rounded-2xl shadow-xl p-6"
      >
        <slot name="main">
          <h2 class="text-xl font-semibold mb-4">Default Main Content</h2>
          <p>This is the default content for the main slot.</p>
        </slot>
      </div>

      <!-- Info Content (less than main) -->
      <div
          class="col-span-1 lg:col-span-4 h-[80vh] overflow-auto bg-white rounded-2xl shadow-xl p-6"
      >
        <slot name="info">
          <h2 class="text-xl font-semibold mb-4">Default Info Content</h2>
          <p>This is the default content for the info slot.</p>
        </slot>
      </div>
    </div>
  </div>
</template>

<script setup>
// Clean and ready
</script>

<style scoped>
::-webkit-scrollbar {
  width: 8px;
}

::-webkit-scrollbar-thumb {
  background-color: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
}
</style>