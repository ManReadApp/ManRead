<script lang="ts" setup>
defineProps<{
    small: boolean;
}>();
</script>

<template>
    <div class="flex min-h-[900px] flex-col bg-gray-50">
        <div
            class="flex min-h-full flex-1 flex-col justify-center py-12 sm:px-6 lg:px-8"
        >
            <div
                class="mt-10 sm:mx-auto sm:w-full"
                :class="small ? 'sm:max-w-[380px]' : 'sm:max-w-[480px]'"
            >
                <div
                    class="bg-white px-6 py-12 shadow-sm sm:rounded-lg sm:px-12"
                >
                    <slot />
                </div>
            </div>
        </div>
    </div>
</template>
