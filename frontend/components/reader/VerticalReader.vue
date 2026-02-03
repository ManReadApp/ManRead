<script setup lang="ts">
import type {Cache, Ret} from "~/utils/img";
import VerticalImage from "~/components/reader/VerticalImage.vue";

//TODO: render next invisible image
defineProps({
  mode: {
    type: String as () => 'v' | 'h',
    required: true
  },
  direction: {type: String as () => 'l2r' | 'r2l', required: true},
  images: {
    type: Array as () => Ret[],
    required: true
  },
  cache: {
    type: Object as () => Cache,
    required: true,
  },
})
//TODO: add id to for loop
</script>

<template>
  <div class="w-full h-full relative">
    <div
        class="absolute"
        :class="{
    'w-full left-0': mode === 'v',
    'h-full top-0 flex': mode === 'h',
    'flex-row-reverse': direction === 'r2l',
  }"
        :style="mode === 'v'
    ? `top: ${images.length ? images[0].offset : 0}px`
    : ((direction === 'l2r' ? 'left' : 'right') + `: ${images.length ? images[0].offset : 0}px`)"
    >
      <VerticalImage
          v-if="mode === 'v'"
          v-for="item in images"
          :data="item"
          :cache="cache"
      />
    </div>
  </div>
</template>

<style scoped>

</style>