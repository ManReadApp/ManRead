<script lang="ts" setup>
defineProps<{ message: string }>();
</script>

<template>
    <div class="sm:mx-auto sm:w-full sm:max-w-md mb-4">
        <NuxtImg
            alt="ManRead"
            class="mx-auto h-40 w-auto"
            draggable="false"
            src="/favicon/favicon.svg"
        />
        <h2
            class="mt-6 text-center text-2xl/9 font-bold tracking-tight text-gray-900 cursor-default"
        >
            {{ message }}
        </h2>
    </div>
</template>

<style scoped></style>
