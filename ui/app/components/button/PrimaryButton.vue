<script lang="ts" setup>
const props = defineProps<{
    name: string;
    enabled: boolean;
}>();

const emits = defineEmits(["click"]);

function click() {
    if (!props.enabled) return;
    emits("click");
}
</script>

<template>
    <button
        class="flex w-full justify-center rounded-md px-3 py-1.5 text-sm/6 font-semibold text-white shadow-xs"
        :class="
            !enabled
                ? 'bg-gray-300'
                : 'bg-indigo-600 hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600'
        "
        type="submit"
        @click="click"
    >
        {{ name }}
    </button>
</template>

<style scoped></style>
