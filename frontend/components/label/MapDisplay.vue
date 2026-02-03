<template>
  <div class="space-y-2">
    <div v-for="(values, key) in data" :key="key"  class="flex items-center space-x-1">
      <div v-if="values.length !== 0" class="text-gray-700">
        {{ key }}:
      </div>
      <TagRowDisplay v-if="values.length !== 0" :values="values" @tag:dblclick="(index) => dblclickValue(key, index)"
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
    dblclickValue(key, value) {
      this.$emit('tag:dblclick', {key, value});
    },
    removeValue(key, value) {
      this.$emit('tag:remove', {key, value});
    }
  }
};
</script>