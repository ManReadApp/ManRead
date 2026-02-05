<script lang="ts" setup>
defineProps<{
    small: boolean;
}>();
</script>

<template>
    <div
        class="flex h-screen flex-col bg-gradient-to-br from-slate-50 via-white to-indigo-50 dark:from-slate-950 dark:via-slate-900 dark:to-indigo-950"
    >
        <div class="flex h-screen flex-1 items-center justify-center px-4 py-10">
            <div
                class="w-full"
                :class="small ? 'sm:max-w-[380px]' : 'sm:max-w-[480px]'"
            >
                <div
                    class="no-scrollbar max-h-[calc(100vh-3rem)] overflow-y-auto rounded-2xl border border-slate-200/70 bg-white/80 px-6 py-10 shadow-xl shadow-slate-200/40 backdrop-blur dark:border-slate-800 dark:bg-slate-900/80 dark:shadow-black/40 sm:px-12"
                >
                    <slot />
                </div>
            </div>
        </div>
    </div>
</template>
