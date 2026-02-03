<script lang="ts">
export default {
  props: {
    values: {
      type: Array,
      required: true
    }
  },
  emits: ["tag:dblclick", "tag:remove"],
  methods: {
    dblclickValue(index: number) {
      this.$emit('tag:dblclick', index);
    },
    removeValue(index: number) {
      this.$emit('tag:remove', index);
    }
  }
};
</script>

<template>
  <div class="flex flex-wrap items-center space-y-1">
    <div
        v-for="(value, index) in values"
        :key="index"
        class="flex items-center space-x-2 mr-1 bg-gray-200 px-1 py-0.5 rounded-sm text-gray-700 text-xs select-none"
    >
          <span @dblclick="dblclickValue(index)">
            {{ value }}
          </span>
      <button
          class="text-gray-500 hover:text-red-500 cursor-pointer"
          @click="removeValue(index)"
          :title="'Double-click to remove ' + value"
      >
            <span class="text-xs font-bold">
              ×
            </span>
      </button>
    </div>
  </div>
</template>

<style scoped>

</style>