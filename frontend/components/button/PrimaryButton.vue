<script lang="ts">
export default {
    props: {
        name: {
            type: String,
            required: true,
        },
        enabled: {
            type: Boolean,
            required: true,
        },
    },
    emits: ["click"],
    methods: {
        click() {
            if (!this.enabled) return;
            this.$emit("click");
        },
    },
};
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
