<script setup lang="ts">
type Chapter = {
  chapter: number,
  chapter_id: string,
  release_date?: string | null | undefined,
  sources: string[],
  titles: string[],
  versions: { [p: string]: string }
};
const props = defineProps({
  manga_id: {
    type: String,
    required: true
  },
  titles: {
    type: Object as () => { [key: string]: {items: string[]} },
    required: true
  },
  chapter: Object as () => Chapter,
})

const getChapterName = computed(() => {
  if (!props.chapter) return;
  if (props.chapter.titles.length == 0 || props.chapter.titles[0] == String(props.chapter.chapter)) return null;
  return props.chapter
})
import {getTitle} from "~/utils/titles";
</script>

<template>
  <div class="absolute top-0 left-0 bg-gray-900 opacity-90 w-full h-12 flex text-white justify-center select-none">
    <ol class="inline-flex items-center space-x-1 md:space-x-2 rtl:space-x-reverse">
      <li class="inline-flex items-center">
        <NuxtLink to="/" class="inline-flex items-center text-sm font-medium hover:text-gray-400">
          <svg class="w-3 h-3 me-2.5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor"
               viewBox="0 0 20 20">
            <path
                d="m19.707 9.293-2-2-7-7a1 1 0 0 0-1.414 0l-7 7-2 2a1 1 0 0 0 1.414 1.414L2 10.414V18a2 2 0 0 0 2 2h3a1 1 0 0 0 1-1v-4a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v4a1 1 0 0 0 1 1h3a2 2 0 0 0 2-2v-7.586l.293.293a1 1 0 0 0 1.414-1.414Z"/>
          </svg>
          Home
        </NuxtLink>
      </li>
      <li>
        <div class="flex items-center">
          <svg class="rtl:rotate-180 w-3 h-3 mx-1" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none"
               viewBox="0 0 6 10">
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="m1 9 4-4-4-4"/>
          </svg>
          <button @click="() => navigateTo(`/info/${manga_id}`)"
                  class="cursor-pointer ms-1 text-sm font-medium  md:ms-2 hover:text-gray-400">{{
              getTitle(titles)
            }}
          </button>
        </div>
      </li>
      <li v-if="chapter">
        <div class="flex items-center cursor-default">
          <svg class="rtl:rotate-180 w-3 h-3  mx-1" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none"
               viewBox="0 0 6 10">
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="m1 9 4-4-4-4"/>
          </svg>
          <span v-if="chapter && getChapterName" class="ms-1 text-sm font-medium md:ms-2">{{
              chapter.chapter
            }}. {{ chapter.titles[0] }}</span>
          <span v-else-if="chapter" class="ms-1 text-sm font-medium md:ms-2 ">{{ chapter.chapter }}</span>
        </div>
      </li>
    </ol>
  </div>
</template>

<style scoped>

</style>
