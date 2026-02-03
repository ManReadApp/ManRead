<script setup lang="ts">
import {type Cache, type DataObject} from "~/utils/img";



const props = defineProps({
  cache: {
    type: Object as () => Cache,
    required: true,
  },
  kind: {
    type: String as PropType<"l" | "r" | "c">,
    required: true,
  },
  data: {
    type: [String, Object] as PropType<"Unreachable" | "Unreachable2" | "Loading" | DataObject>,
    required: true,
  },
});

defineEmits(['click:image'])


function getUrl(data: DataObject) {
  const key = data.chapter_id + data.complete!.version_id + data.complete!.page.page;
  return props.cache[key]
}

const r = computed(() => {
  if (typeof props.data === 'string') {
    return null;
  }
  return getUrl(props.data)
});

</script>

<template>
  <div v-if="data === 'Loading'" class="w-full h-full">Loading...</div>
  <div v-else-if="data === 'Unreachable'" class="w-full h-full">Unreachable</div>
  <div v-else-if="data === 'Unreachable2'" class="w-full h-full">Unreachable2</div>
  <div v-else-if="!data.complete" class="w-full h-full">No Data</div>
  <div v-else class="w-full h-full">
    <div v-if="!r">
      Loading Image...
    </div>
    <div v-else-if="r.value.error">Failed loading image: {{ r.value.error }}</div>
    <div v-else-if="r.value.success" class="w-full h-full" :class="{'img-l': kind =='l',
    'img-r': kind =='r', 'img-c': kind == 'c'}"><img :id="data.complete.page.id" :src="r.value.success" alt="Manga Image" @click="$emit('click:image', $event)"/></div>
    <div v-else>
    </div>
  </div>
</template>

<style scoped>
img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.img-r img {
  object-position: right;
}
.img-l img {
  object-position: left;
}

.img-c {
  object-position: center;
}

</style>