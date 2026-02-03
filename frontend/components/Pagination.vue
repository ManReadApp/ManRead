<script lang="ts">
export default {
  props: {
    last_page: {
      type: Number,
      required: true,
    },
    emit: ['change-page'],
    current_page: {
      type: Number,
      required: true,
    }
  },
  methods: {
    generatePagination(n: number, end: number, current: number) {
      if (n % 2 === 0) n += 1;
      if (end <= n) return Array.from({length: end}, (_, i) => i + 1);

      const pages = new Set([1, end]);
      let generateN = Math.floor((n - 3) / 2);

      let start = Math.max(2, current - generateN);
      if (start <= current) {
        for (let i = start; i <= current; i++) {
          pages.add(i);
        }
      }

      const firstValue = current + 1;
      if (end >= firstValue) {
        let count = 0;
        generateN = (n - pages.size);
        for (let i = firstValue; i <= end && count < generateN; i++) {
          pages.add(i);
          count++;
        }
      }

      const sortedPages = [...pages].sort((a, b) => a - b);

      const result = [];
      let prev = null;

      for (const page of sortedPages) {
        if (prev !== null && page !== prev + 1) {
          result.push("...");
        }
        result.push(page);
        prev = page;
      }

      return result;
    }
  }
}

</script>

<template>
  <div class="mx-auto max-w-7xl py-8 sm:px-6 lg:px-8 w-full">
    <nav
        class="flex items-center justify-between border-t border-gray-200 px-4 sm:px-0"
    >
      <div class="-mt-px flex w-0 flex-1">
        <a
            class="inline-flex items-center border-t-2 border-transparent pt-4 pr-1 text-sm font-medium text-gray-500"
            :class="{'hover:border-gray-300 hover:text-gray-700 cursor-pointer': current_page > 1,'cursor-not-allowed':current_page <= 1} "
            @click="() => current_page > 1 ? $emit('change-page', current_page-1) :null"
        >
          <svg
              aria-hidden="true"
              class="mr-3 size-5 text-gray-400"
              data-slot="icon"
              fill="currentColor"
              viewbox="0 0 20 20"
              xmlns="http://www.w3.org/2000/svg"
          >
            <path
                clip-rule="evenodd"
                d="M18 10a.75.75 0 0 1-.75.75H4.66l2.1 1.95a.75.75 0 1 1-1.02 1.1l-3.5-3.25a.75.75 0 0 1 0-1.1l3.5-3.25a.75.75 0 1 1 1.02 1.1l-2.1 1.95h12.59A.75.75 0 0 1 18 10Z"
                fill-rule="evenodd"
            ></path>
          </svg
          >
          Previous</a
        >
      </div>
      <div class="hidden md:-mt-px md:flex">

        <div v-for="item in generatePagination(7, last_page, current_page)"
             class="inline-flex items-center border-t-2 border-transparent px-4 pt-4 text-sm font-medium text-gray-500"
             @click="() => (item != current_page && item != '...') ? $emit('change-page', item) :null"
             :class="{'cursor-default border-indigo-500 text-indigo-600': item == current_page, 'cursor-pointer hover:border-gray-300 hover:text-gray-700': item != current_page && item != '...', 'cursor-default': item == '...'}"
        >{{ item }}
        </div>
      </div>
      <div class="-mt-px flex w-0 flex-1 justify-end">
        <div
            class="inline-flex items-center border-t-2 border-transparent pt-4 pl-1 text-sm font-medium text-gray-500 "
            :class="{'hover:border-gray-300 hover:text-gray-700 cursor-pointer': current_page < last_page,'cursor-not-allowed':current_page >= last_page} "
            @click="() => current_page < last_page ? $emit('change-page', current_page+1) :null"
        >Next
          <svg
              aria-hidden="true"
              class="ml-3 size-5 text-gray-400"
              data-slot="icon"
              fill="currentColor"
              viewbox="0 0 20 20"
              xmlns="http://www.w3.org/2000/svg"
          >
            <path
                clip-rule="evenodd"
                d="M2 10a.75.75 0 0 1 .75-.75h12.59l-2.1-1.95a.75.75 0 1 1 1.02-1.1l3.5 3.25a.75.75 0 0 1 0 1.1l-3.5 3.25a.75.75 0 1 1-1.02-1.1l2.1-1.95H2.75A.75.75 0 0 1 2 10Z"
                fill-rule="evenodd"
            ></path>
          </svg
          >
        </div>
      </div>
    </nav>
  </div>
</template>

<style scoped>

</style>