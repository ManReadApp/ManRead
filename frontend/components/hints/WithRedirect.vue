<script setup lang="ts">

</script>

<template>
  <div class="bg-white">
    <div class="mx-auto max-w-7xl px-4 py-12 sm:px-6 lg:px-8">
      <div class="mx-auto max-w-4xl">
        <div class="rounded-md bg-blue-50 p-4">
          <div class="flex">
            <div class="shrink-0">
              <svg
                  aria-hidden="true"
                  class="size-5 text-blue-400"
                  data-slot="icon"
                  fill="currentColor"
                  viewbox="0 0 20 20"
                  xmlns="http://www.w3.org/2000/svg"
              >
                <path
                    clip-rule="evenodd"
                    d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Zm-7-4a1 1 0 1 1-2 0 1 1 0 0 1 2 0ZM9 9a.75.75 0 0 0 0 1.5h.253a.25.25 0 0 1 .244.304l-.459 2.066A1.75 1.75 0 0 0 10.747 15H11a.75.75 0 0 0 0-1.5h-.253a.25.25 0 0 1-.244-.304l.459-2.066A1.75 1.75 0 0 0 9.253 9H9Z"
                    fill-rule="evenodd"
                ></path>
              </svg>
            </div>
            <div class="ml-3 flex-1 md:flex md:justify-between">
              <p class="text-sm text-blue-700">
                A new software update is available. See what’s new
                in version 2.0.4.
              </p>
              <p class="mt-3 text-sm md:mt-0 md:ml-6">
                <a
                    class="font-medium whitespace-nowrap text-blue-700 hover:text-blue-600"
                    href="#"
                >Details<span aria-hidden="true"> →</span></a
                >
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

</template>

<style scoped>

</style>