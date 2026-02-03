<template>
  <div :class="previewUrl ? 'h-64' : 'h-10'" class="w-full transition-all duration-300">
    <label
        for="image-upload"
        class="w-full h-full flex flex-col items-center justify-center border-1 border-dashed border-gray-300 rounded-lg cursor-pointer transition hover:border-blue-400 hover:bg-blue-50"
    >
      <input
          id="image-upload"
          type="file"
          accept="image/*"
          class="hidden"
          @change="handleFileChange"
      />
      <div v-if="!previewUrl" class="flex flex-col items-center">
        <span class="text-gray-500 text-sm">Click to upload an image</span>
      </div>

      <img
          v-else
          :src="previewUrl"
          alt="Image preview"
          class="max-h-full max-w-full object-contain rounded"
      />
    </label>
  </div>
</template>

<script setup>
import {ref} from 'vue'
import {useFetch} from '#app'

const emit = defineEmits(['uploaded'])

const previewUrl = ref(null)

async function handleFileChange(event) {
  const file = event.target.files[0]

  if (file && file.type.startsWith('image/')) {
    const reader = new FileReader()
    reader.onload = (e) => {
      previewUrl.value = e.target.result
    }
    reader.readAsDataURL(file)

    const formData = new FormData()
    formData.append('file', file)

    const {data, error} = await useFetch('/api/v1/image/upload', {
      method: 'POST',
      body: formData,
    })

    if (
        error.value ||
        !data.value ||
        !Array.isArray(data.value) ||
        !Array.isArray(data.value[0]) ||
        data.value[0].length < 2
    ) {
      emit("uploaded", null);
    } else {
      emit("uploaded", data.value[0][1]);
    }
  } else {
    previewUrl.value = null
    emit("uploaded", null);
  }
}
</script>