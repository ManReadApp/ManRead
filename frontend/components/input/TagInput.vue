<script setup lang="ts">
type Tag = {
  tag: string;
  description: string | undefined;
  sex: TagSex
}

enum TagSex {
  Female = "Female",
  Male = "Male",
  Both = "Both",
  None = "None",
  FemaleMale = "FemaleMale",
  MaleFemale = "MaleFemale",
  Unknown = "Unknown",
}


const props = defineProps<{
  sexOptions: string[];
  tags: Tag[];
  searchFn?: (query: string, sex: TagSex) => Promise<Tag[]>;
}>();

const searchQuery = ref('');
const selectedSex = ref<TagSex>(TagSex.None);
const searchResults = ref<Tag[]>([]);
const isSearching = ref(false);

const performSearch = async () => {
  if (!props.searchFn) return;

  isSearching.value = true;
  try {
    searchResults.value = await props.searchFn(searchQuery.value, selectedSex.value);
  } finally {
    isSearching.value = false;
  }
};

watch([searchQuery, selectedSex], () => {
  if (searchQuery.value.length > 2) performSearch();
});

const addTag = (tag: Tag) => {
  if (!props.tags.some(t => t.tag === tag.tag && t.sex === tag.sex)) {
    props.tags.push(tag);
  }
  searchQuery.value = '';
};
</script>
<template>
  <div class="space-y-2">
    <label class="block text-sm font-medium text-gray-700">Label</label>

    <div class="flex gap-2">
      <input
          v-model="searchQuery"
          class="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-400 outline-none transition-all"
          placeholder="Search tags"
      >
      <select
          v-model="selectedSex"
          class="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-400 outline-none transition-all text-sm"
      >
        <option v-for="option in sexOptions" :value="option">
          {{ option }}
        </option>
      </select>
    </div>

    <div v-if="isSearching" class="text-sm text-gray-500 p-2">Searching tags...</div>
    <div v-else class="absolute z-50 w-full mt-1 shadow-lg">
      <div class="bg-white border border-gray-200 rounded-lg max-h-60 overflow-y-auto">
        <div
            v-for="result in searchResults"
            :key="`${result.tag}-${result.sex}`"
            @click="addTag(result)"
            class="px-4 py-2 hover:bg-blue-50 cursor-pointer text-sm text-gray-700 transition-colors"
        >
          <span class="font-medium">{{ result.tag }}</span>
          <span class="text-gray-400 ml-2 text-xs">{{ TagSex[result.sex] }}</span>
        </div>
      </div>
    </div>

    <div class="flex flex-wrap gap-2 mt-2">
      <div
          v-for="(tag, index) in tags"
          :key="index"
          class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-700"
      >
        {{ tag.tag }}
        <span class="text-gray-400 ml-1 text-xs">{{ TagSex[tag.sex] }}</span>
        <button
            @click="tags.splice(index, 1)"
            class="ml-1.5 text-gray-500 hover:text-gray-700"
        >
          ×
        </button>
      </div>
    </div>
  </div>
</template>