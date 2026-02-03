<script setup lang="ts">
import {type Cache, type DataObject, isDataObject, type Ret} from "~/utils/img";
import {useElementSize} from "@vueuse/core";


const props = defineProps({
  cache: {
    type: Object as () => Cache,
    required: true,
  },
  data: {
    type: Object as () => Ret,
    required: true,
  },
});


function getUrl(data: DataObject) {
  const key = data.chapter_id + data.complete!.version_id + data.complete!.page.page;
  return props.cache[key]
}

const r = computed(() => {
  if (!(isDataObject(props.data))) {
    return null;
  }
  return getUrl(props.data)
});

const complete = computed(() => {
  if (!(isDataObject(props.data))) {
    return null;
  }
  return props.data.complete;
})

const message = computed(() => {
  if (!(isDataObject(props.data))) {
    return props.data?.message
  }
  return null;
})
const containerRef = ref(null);
const containerSize = useElementSize(containerRef)
const height = computed(() => {
  if (!props.data || !(isDataObject(props.data)) || !props.data.complete) {
    return containerSize.width.value;
  }
  const page = props.data.complete.page;
  return containerSize.width.value * page.height / page.width
})


</script>

<template>
  <div ref="containerRef" class="w-full" :style="`height: ${height}px`">
    <p v-if="message === 'Loading'">Loading...</p>
    <p v-else-if="message === 'Unreachable'">Unreachable</p>
    <p v-else-if="message === 'Unreachable2'">Unreachable2</p>
    <p v-else-if="!complete">No Data</p>
    <template v-else>
      <p v-if="!r">
        Try Loading Image...
      </p>
      <p v-else-if="r.value.error">Failed loading image: {{ r.value.error }}</p>
      <img v-else-if="r.value.success" :id="complete.page.id" :src="r.value.success"
           alt="Manga Image"/>
      <p v-else>
        Loading Image...
      </p>
    </template>
  </div>

</template>

<style scoped>
img {
  width: 100%;
  filter: blur(var(--image-blur)) brightness(var(--image-brightness)) contrast(var(--image-contrast)) grayscale(var(--image-grayscale)) hue-rotate(var(--image-hue-rotate)) invert(var(--image-invert)) saturate(var(--image-saturate)) sepia(var(--image-sepia));
}
</style>