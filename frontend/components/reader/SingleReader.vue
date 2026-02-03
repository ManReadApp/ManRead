<script lang="ts">
import type {Cache, DataObject} from "~/utils/img";
import FullImage from "~/components/reader/FullImage.vue";

export default {
  components: {FullImage},
  props: {
    cache: {
      type: Object as () => Cache,
      required: true,
    },
    leftToRight: {
      type: Boolean,
      required: true
    },
    items: {
      type: Array as () => Array<"Unreachable" | "Unreachable2" | "Loading" | DataObject>,
      required: true
    }
  },
  methods: {
    handleClick(event: any) {
      const image = event.target;
      const rect = image.getBoundingClientRect();
      const clickX = event.clientX;
      const item =  this.items[0];
      if (typeof item === "string")return;
      if(!item.complete)return
      if (this.leftToRight) {
        if (clickX < rect.left + rect.width / 4) {

          this.$emit("jump-to-page",item.complete.page.page - 1 )
        } else {
          this.$emit("jump-to-page",item.complete.page.page + 1)
        }
      } else {
        if (clickX < rect.left + rect.width / 4 * 3) {
          this.$emit("jump-to-page", item.complete.page.page + 1)
        } else {
          this.$emit("jump-to-page", item.complete.page.page - 1)
        }
      }
    },
  },
  computed: {
    f_img() {
      return this.items[0]
    }
  },
  emits: ["jump-to-page"]
}

</script>

<template>
  <FullImage v-if="f_img" :data="f_img" kind="c" :cache="cache" @click:image="handleClick"/>
  <div v-else>No Images</div>
</template>

<style scoped>

</style>