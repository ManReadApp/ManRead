<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{ message: string; color: string; body?: string }>();

const styles = computed(() => {
    switch (props.color) {
        case "green":
            return {
                container:
                    "bg-emerald-50 text-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-100",
                title: "text-emerald-800 dark:text-emerald-100",
                body: "text-emerald-700 dark:text-emerald-200",
                icon: "text-emerald-400",
            };
        case "yellow":
            return {
                container:
                    "bg-amber-50 text-amber-900 dark:bg-amber-950/40 dark:text-amber-100",
                title: "text-amber-800 dark:text-amber-100",
                body: "text-amber-700 dark:text-amber-200",
                icon: "text-amber-400",
            };
        default:
            return {
                container:
                    "bg-slate-50 text-slate-900 dark:bg-slate-900/60 dark:text-slate-100",
                title: "text-slate-800 dark:text-slate-100",
                body: "text-slate-600 dark:text-slate-200",
                icon: "text-slate-400",
            };
    }
});
</script>

<template>
    <div class="rounded-md p-4" :class="styles.container">
        <div class="flex">
            <div class="shrink-0">
                <svg
                    v-if="color === 'green'"
                    aria-hidden="true"
                    class="size-5"
                    :class="styles.icon"
                    data-slot="icon"
                    fill="currentColor"
                    viewbox="0 0 20 20"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path
                        clip-rule="evenodd"
                        d="M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16Zm3.857-9.809a.75.75 0 0 0-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 1 0-1.06 1.061l2.5 2.5a.75.75 0 0 0 1.137-.089l4-5.5Z"
                        fill-rule="evenodd"
                    />
                </svg>
                <svg
                    v-if="color === 'yellow'"
                    aria-hidden="true"
                    class="size-5"
                    :class="styles.icon"
                    data-slot="icon"
                    fill="currentColor"
                    viewbox="0 0 20 20"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path
                        clip-rule="evenodd"
                        d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495ZM10 5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 10 5Zm0 9a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z"
                        fill-rule="evenodd"
                    />
                </svg>
            </div>
            <div class="ml-3">
                <h3 class="text-sm font-medium" :class="styles.title">
                    {{ message }}
                </h3>
                <div
                    v-if="body"
                    class="mt-2 text-sm"
                    :class="styles.body"
                >
                    <p>
                        <slot />
                    </p>
                </div>
            </div>
        </div>
    </div>
</template>
