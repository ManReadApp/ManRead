<script lang="ts">
import QOI from 'qoijs'

export default {
  props: {
    manga_id: {type: String, required: true},
    ext: {type: String, required: true},
  },
  methods: {
    async load_img() {
      const blob: Blob = await $fetch('http://localhost:8082/api/v1/image/cover', {
        method: "POST",
        body: {
          manga_id: this.manga_id,
          file_ext: this.ext,
        },
        responseType: 'blob',
        headers: {Authorization: `Bearer ${await getAccessToken()}`}
      });
      if (this.ext == 'qoi') {
        const arrayBuffer = await blob.arrayBuffer();
        const qoiImage = QOI.decode(new Uint8Array(arrayBuffer)); // Decode QOI

        const canvas = document.createElement("canvas");
        const ctx = canvas.getContext("2d");

        if (ctx) {
          canvas.width = qoiImage.width;
          canvas.height = qoiImage.height;
          const imageData = new ImageData(new Uint8ClampedArray(qoiImage.data.buffer), qoiImage.width, qoiImage.height);
          ctx.putImageData(imageData, 0, 0);
        }

        this.data = canvas.toDataURL("image/png")
      } else {
        this.data =  URL.createObjectURL(blob);
      }
    }
  },
  mounted() {
    this.load_img()
  },
  setup() {
    return {
      data: ref<string>("")
    }
  }
}
</script>

<template>
  <img v-if="data" :src="data" alt="" class="w-full h-full"/>
</template>

<style scoped>

</style>