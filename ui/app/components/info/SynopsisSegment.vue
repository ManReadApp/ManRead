<script setup lang="ts">
import Tag from "~/components/info/Tag.vue";

defineProps<{ label: string; items: { label: string; href: string }[] }>();
</script>

<template>
    <div class="font-bold mb-2">{{ label }}</div>
    <div class="flex gap-2 flex-wrap max-w-[26rem]">
        <Tag
            v-for="item in items"
            :key="item.href"
            :href="item.href"
            :label="item.label"
        />
    </div>
</template>

<style scoped></style>
