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
        class="flex w-full justify-center rounded-md px-3 py-2 text-sm/6 font-semibold text-white shadow-sm transition-colors"
        :class="
            !enabled
                ? 'bg-slate-300 text-slate-500 dark:bg-slate-700 dark:text-slate-300'
                : 'bg-indigo-600 hover:bg-indigo-500 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500 focus-visible:ring-offset-2 focus-visible:ring-offset-white dark:focus-visible:ring-offset-slate-950'
        "
        type="submit"
        @click="click"
    >
        {{ name }}
    </button>
</template>

<style scoped></style>
