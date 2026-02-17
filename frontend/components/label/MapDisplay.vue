<template>
  <div class="space-y-2">
    <div v-for="(entry, key) in data" :key="key"  class="flex items-center space-x-1">
      <div v-if="getItems(entry).length !== 0" class="text-gray-700">
        {{ key }}:
      </div>
      <TagRowDisplay v-if="getItems(entry).length !== 0" :values="getItems(entry)" @tag:dblclick="(index) => dblclickValue(key, index)"
                     @tag:remove="(index) => removeValue(key, index)"/>
    </div>
  </div>
</template>

<script>
import TagRowDisplay from "~/components/label/TagRowDisplay.vue";
export default {
  components: {TagRowDisplay},
  props: {
    data: {
      type: Object,
      required: true
    }
  },
  emits: ["tag:dblclick", "tag:remove"],
  methods: {
    getItems(entry) {
      if (Array.isArray(entry)) return entry;
      return entry?.items ?? [];
    },
    dblclickValue(key, value) {
      this.$emit('tag:dblclick', {key, value});
    },
    removeValue(key, value) {
      this.$emit('tag:remove', {key, value});
    }
  }
};
</script>
