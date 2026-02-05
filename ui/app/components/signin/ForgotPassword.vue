<template>
    <div class="flex items-center justify-between">
        <div />
        <div class="text-sm/6">
            <NuxtLink
                class="font-semibold text-indigo-600 hover:text-indigo-500"
                :to="to"
                >{{ name }}</NuxtLink
            >
        </div>
    </div>
</template>
<script lang="ts" setup>
defineProps<{
    name: string;
    to: string;
}>();
</script>
