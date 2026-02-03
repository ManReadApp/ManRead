<script setup lang="ts">

import IconButton from "~/components/reader/nav/IconButton.vue";
import {useElementBounding} from "@vueuse/core";
import Settings from "~/components/reader/nav/Settings.vue";

type Chapter = {
  chapter: number,
  chapter_id: string,
  release_date?: string | null | undefined,
  sources: string[],
  titles: string[],
  versions: { [p: string]: string }
};
defineProps({
  progress: {
    required: true,
    value: Number as () => number,
  },
  chapters: Array as () => Array<Chapter>,
  scrollbottom: {
    required: true,
    value: Boolean
  },
})

const readerMode = defineModel<'v' | 'h' | 's' | 'd'>('readerMode', {required: true})
const sizeMode = defineModel<'l' | '%' | 'm'>('sizeMode', {required: true})


const readDirection = defineModel('readDirection', {required: true})

const isFullscreen = ref(false);
const fullscreenAvailable = ref(false);
const chapterRef = ref(null);
const settingsRef = ref(null);

const chapterBounding = useElementBounding(chapterRef)
const settingsBounding = useElementBounding(settingsRef)

const chapterContainerActive = ref(false);
const settingsContainerActive = ref(false);

onMounted(() => {
  isFullscreen.value = !!document.fullscreenElement;
  fullscreenAvailable.value = document.fullscreenEnabled;
});

const toggleFullscreen = () => {
  if (!document.fullscreenElement) {
    document.documentElement.requestFullscreen();
    isFullscreen.value = true;
  } else if (document.exitFullscreen) {
    document.exitFullscreen();
    isFullscreen.value = false;
  }
};
const imageSettings = defineModel('imageSettings', {required: true})

defineEmits(["clicked:last-chapter", "clicked:next-chapter", "clicked:auto-scroll", 'clicked:open-sizing'])
</script>

<template>
  <div v-if="scrollbottom" class="absolute bottom-0 left-0 bg-gray-900 opacity-90 w-full h-14 flex justify-around">
    <div class="absolute inset-0 h-1 -translate-y-1 bg-orange-400" :style="`width: ${progress}%`"></div>
    <IconButton v-if="readerMode == 'v' || readerMode=='h'" label="Auto Scroll" class="group"
                @click="$emit('clicked:auto-scroll')">
      <svg class="h-full cursor-pointer lucide lucide-chevrons-down" viewBox="0 0 24 24" fill="none"
           stroke="#fff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
      >
        <g class="opacity-60 group-hover:opacity-100 transition-opacity duration-100">
          <path d="m7 6 5 5 5-5"/>
          <path d="m7 13 5 5 5-5"/>
        </g>
      </svg>
    </IconButton>
    <IconButton label="Back" class="group" @click="$emit('clicked:last-chapter')">
      <svg viewBox="0 0 24 24" fill="none"
           stroke="#fff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
           class="h-full cursor-pointer lucide lucide-chevron-left">
        <g class="opacity-60 group-hover:opacity-100 transition-opacity duration-100">

          <path d="m15 18-6-6 6-6"/>
        </g>
      </svg>
    </IconButton>
    <IconButton label="Next" class="group" @click="$emit('clicked:next-chapter')">
      <svg viewBox="0 0 24 24" fill="none"
           stroke="#fff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
           class="h-full cursor-pointer lucide lucide-chevron-right">
        <g class="opacity-60 group-hover:opacity-100 transition-opacity duration-100">

          <path d="m9 18 6-6-6-6"/>
        </g>
      </svg>
    </IconButton>
    <IconButton ref="chapterRef" label="Chapters" @click="() =>chapterContainerActive = !chapterContainerActive"
                class="group">
      <svg class="h-full cursor-pointer" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg">
        <g class="opacity-60 group-hover:opacity-100 transition-opacity duration-100">
          <path d="M7 7H20" stroke="white" stroke-width="2" stroke-linecap="round"/>
          <path d="M7 14H20" stroke="white" stroke-width="2" stroke-linecap="round"/>
          <path d="M7 21H20" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </g>
      </svg>
    </IconButton>
    <IconButton v-if="fullscreenAvailable" label="Fullscreen" @click="toggleFullscreen" class="group">
      <svg v-if="isFullscreen" class="h-full cursor-pointer" viewBox="0 0 28 28" fill="none"
           xmlns="http://www.w3.org/2000/svg">
        <g class="opacity-60 group-hover:opacity-100 transition-opacity duration-100">
          <rect opacity="0.01" x="0.466667" y="0.466667" width="27.0667" height="27.0667" fill="#D8D8D8"
                stroke="#979797" stroke-width="0.933333"/>
          <path d="M 5.5028 9.2362 L 8.3028 9.2362 C 8.8183 9.2362 9.2362 8.8183 9.2362 8.3028 V 5.5028"
                stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M 18.065 5.5028 L 18.065 8.3028 C 18.065 8.8183 18.4828 9.2362 18.9983 9.2362 L 21.7983 9.2362"
                stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M 9.2362 21.7983 L 9.2362 18.9983 C 9.2362 18.4828 8.8183 18.065 8.3028 18.065 L 5.5028 18.065"
                stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path
              d="M 18.065 21.7983 L 18.065 18.9983 C 18.065 18.4828 18.4828 18.065 18.9983 18.065 L 21.7983 18.065"
              stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </g>
      </svg>
      <svg v-else class="h-full cursor-pointer" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg">
        <g class="opacity-60 group-hover:opacity-100 transition-opacity duration-100">
          <rect opacity="0.01" x="0.466667" y="0.466667" width="27.0667" height="27.0667" fill="#D8D8D8"
                stroke="#979797" stroke-width="0.933333"/>
          <path d="M9.3349 5.60156L6.5349 5.60156C6.01943 5.60156 5.60156 6.01943 5.60156 6.5349V9.3349"
                stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M22.3984 9.3349L22.3984 6.5349C22.3984 6.01943 21.9806 5.60156 21.4651 5.60156L18.6651 5.60156"
                stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M5.60156 18.6651L5.60156 21.4651C5.60156 21.9806 6.01943 22.3984 6.5349 22.3984L9.3349 22.3984"
                stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M22.3984 18.6651L22.3984 21.4651C22.3984 21.9806 21.9806 22.3984 21.4651 22.3984L18.6651 22.3984"
                stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </g>
      </svg>
    </IconButton>
    <IconButton ref="settingsRef" label="Settings" @click="() =>settingsContainerActive = !settingsContainerActive"
                class="group">
      <svg class="h-full cursor-pointer" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg">
        <g class="opacity-60 group-hover:opacity-100 transition-opacity duration-100">
          <rect opacity="0.01" x="0.466667" y="0.466667" width="27.0667" height="27.0667" fill="#D8D8D8"
                stroke="#979797" stroke-width="0.933333"/>
          <path fill-rule="evenodd" clip-rule="evenodd"
                d="M17.7058 5.72009C18.3726 5.72009 18.9889 6.07588 19.3223 6.65343L23.0273 13.0707C23.3608 13.6482 23.3608 14.3598 23.0273 14.9374L19.3223 21.3546C18.9889 21.9322 18.3726 22.288 17.7058 22.288L10.2957 22.288C9.62883 22.288 9.0126 21.9322 8.67915 21.3546L4.97414 14.9374C4.64069 14.3598 4.64069 13.6482 4.97414 13.0707L8.67915 6.65343C9.0126 6.07588 9.62883 5.72009 10.2957 5.72009L17.7058 5.72009Z"
                stroke="white" stroke-width="2" stroke-linejoin="round"/>
          <circle cx="14.207" cy="14.2109" r="2.5" fill="white"/>
        </g>
      </svg>
    </IconButton>
  </div>
  <div v-if="chapterContainerActive || settingsContainerActive" class="absolute inset-0"
       @click="() => {chapterContainerActive = false ;settingsContainerActive = false}"></div>
  <div v-if="chapterContainerActive" class="absolute text-white rounded bg-gray-900"
       :style="`transform: translate(-50%, calc(-100% - 14px)); top: ${chapterBounding.top.value}px; left: ${chapterBounding.left.value+chapterBounding.width.value/2}px;`">
    <div class="overflow-auto" style="max-height: 50vh; min-width: 200px">
      <div class="px-8  py-2 hover:bg-white/10" v-for="chapter in chapters">{{ chapter.chapter }}.
        {{ chapter.titles[0] }}
      </div>
    </div>
  </div>

  <div v-if="settingsContainerActive" class="absolute text-white rounded overflow-hidden bg-gray-900"
       :style="`transform: translate(-100%, calc(-100% - 14px)); top: ${settingsBounding.top.value}px; left: ${settingsBounding.right.value}px;`">
    <div class="overflow-auto" style="max-height: 50vh; min-width: 400px">
      <Settings v-model:reader-mode="readerMode" v-model:read-direction="readDirection"
                :image-settings="imageSettings" v-model:size-mode="sizeMode" @openSizing="$emit('clicked:open-sizing')"/>
    </div>
  </div>
</template>

<style scoped>

</style>