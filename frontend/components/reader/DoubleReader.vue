<script lang="ts">
import type {Cache, DataObject} from "~/utils/img";
import FullImage from "~/components/reader/FullImage.vue";
import SingleReader from "~/components/reader/SingleReader.vue";

export default {
  components: {SingleReader, FullImage},
  emits: ["jump-to-page"],
  props: {
    cache: {
      type: Object as () => Cache,
      required: true,
    },
    leftToRight: {
      required: true,
      type: Boolean,
    },
    items: {
      type: Array as () => Array<"Unreachable" | "Unreachable2" | "Loading" | DataObject>,
      required: true
    }
  },
  methods: {
    goBack() {
      const item = this.items[0];
      if (typeof item === "string") return;
      if (!item.complete) return;
      this.$emit("jump-to-page", item.complete.page.page - 1)
    },
    goForward() {
      const item = this.items[0];
      if (typeof item === "string") return;
      if (!item.complete) return;
      this.$emit("jump-to-page", item.complete.page.page + 2)
    },
  },
  computed: {
    f_img() {
      return this.items[0];
    },
    s_img() {
      const s = this.items[1];
      if (typeof s === "string") return s;
      const f = this.items[0];
      if (typeof f === "string") return s;
      return s ? s.chapter_id == f.chapter_id ? s : null : null;
    }
  }
}
</script>

<template>
  <div v-if="f_img" class="w-full h-full">
    <div v-if="s_img" class="w-full h-full flex">
      <div class="w-1/2 h-full">
        <FullImage :data="leftToRight ? f_img: s_img" kind="r" :cache="cache"
                   @click:image="() => leftToRight ? goBack() : goForward()"/>
      </div>
      <div class=" w-1/2 h-full">
        <FullImage :data="leftToRight ? s_img: f_img" kind="l" :cache="cache"
                   @click:image="() => leftToRight ? goForward(): goBack()"/>
      </div>
    </div>
    <SingleReader :items="items" :left-to-right="leftToRight" :cache="cache"
                  @jump-to-page="$emit('jump-to-page', $event)"/>
  </div>
  <div v-else>No data loaded</div>
</template>

<style scoped>

</style>