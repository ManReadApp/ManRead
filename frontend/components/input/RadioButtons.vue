<template>
    <div class="w-full sm:px-0">
        <div class="flex w-full">
            <div
                v-for="(category, index) in categories"
                :key="category"
                class="text-center"
                :class="[
                    'flex-1 cursor-pointer transition-all border border-gray-300 p-3 text-sm font-medium',
                    index !== 0 && 'ml-[-1px]',
                    selectedCategory === category
                        ? [colors[index], 'text-gray-800', 'z-10']
                        : 'text-gray-700  hover:bg-gray-200',
                    index === 0 ? 'rounded-l-lg' : '',
                    index === categories.length - 1 ? 'rounded-r-lg' : '',
                ]"
                @click="selectedCategory = category"
            >
                {{ category }}
            </div>
        </div>
    </div>
</template>

<script setup>
import { ref, defineProps } from "vue";

const props = defineProps({
    categories: {
        type: Array,
        required: true,
    },
    colors: {
        type: Array,
        default: () => [
            "bg-red-600 border-red-600",
            "bg-blue-600 border-blue-600",
            "bg-green-600 border-green-600",
            "bg-purple-600 border-purple-600",
        ],
    },
});

const selectedCategory = defineModel();
</script>

<style scoped>
/* Remove any custom CSS that might interfere */
</style>
