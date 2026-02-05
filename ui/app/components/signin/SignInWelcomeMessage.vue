<script lang="ts" setup>
defineProps<{ message: string }>();
</script>

<template>
    <div class="sm:mx-auto sm:w-full sm:max-w-md mb-6">
        <NuxtImg
            alt="ManRead"
            class="mx-auto h-28 w-auto"
            draggable="false"
            src="/favicon/favicon.svg"
        />
        <h2
            class="mt-6 text-center text-2xl/9 font-bold tracking-tight text-slate-900 dark:text-slate-100 cursor-default"
        >
            {{ message }}
        </h2>
    </div>
</template>

<style scoped></style>
