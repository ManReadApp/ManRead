<script lang="ts" setup>
import ChevronLeftIcon from "~/components/icons/ChevronLeftIcon.vue";
import HeartIcon from "~/components/icons/HeartIcon.vue";
import EllipsisVerticalIcon from "~/components/icons/EllipsisVerticalIcon.vue";
import { ref, onMounted, onBeforeUnmount } from "vue";

defineProps<{ isFavorite: boolean | undefined }>();
const emit = defineEmits([
    "back",
    "toggleFavorite",
    "delete",
    "edit",
    "addChapter",
    "modifySeasons",
]);

const menuOpen = ref(false);
const menuRef = ref<HTMLElement | null>(null);

const onClickOutside = (event: MouseEvent) => {
    if (!menuRef.value) return;
    if (!menuRef.value.contains(event.target as Node)) {
        menuOpen.value = false;
    }
};

onMounted(() => {
    document.addEventListener("click", onClickOutside);
});

onBeforeUnmount(() => {
    document.removeEventListener("click", onClickOutside);
});
</script>

<template>
    <nav
        class="fixed left-0 top-2 z-10 flex h-0 w-full items-center justify-between bg-transparent p-4"
    >
        <button
            class="-translate-x-3 -translate-y-2 cursor-pointer text-slate-900 dark:text-white"
            @click="emit('back')"
        >
            <ChevronLeftIcon />
        </button>

        <div class="relative flex items-center gap-2" ref="menuRef">
            <button
                class="rounded-md p-2 text-red-500 transition hover:bg-red-500/10"
                title="Favorite"
                @click="emit('toggleFavorite')"
            >
                <HeartIcon :style="isFavorite ? 'fill: red' : ''" />
            </button>
            <button
                class="rounded-md p-2 text-slate-700 transition hover:bg-slate-500/10 dark:text-slate-200"
                title="Actions"
                @click.stop="menuOpen = !menuOpen"
            >
                <EllipsisVerticalIcon />
            </button>

            <div
                v-if="menuOpen"
                class="absolute right-0 top-11 z-20 w-48 rounded-lg border border-slate-200 bg-white/95 p-1 shadow-lg backdrop-blur dark:border-slate-700 dark:bg-slate-900/95"
            >
                <button
                    class="w-full rounded-md px-3 py-2 text-left text-sm text-slate-700 hover:bg-slate-100 dark:text-slate-200 dark:hover:bg-slate-800"
                    @click="emit('edit'); menuOpen = false"
                >
                    Edit
                </button>
                <button
                    class="w-full rounded-md px-3 py-2 text-left text-sm text-slate-700 hover:bg-slate-100 dark:text-slate-200 dark:hover:bg-slate-800"
                    @click="emit('addChapter'); menuOpen = false"
                >
                    Add Chapter
                </button>
                <button
                    class="w-full rounded-md px-3 py-2 text-left text-sm text-slate-700 hover:bg-slate-100 dark:text-slate-200 dark:hover:bg-slate-800"
                    @click="emit('modifySeasons'); menuOpen = false"
                >
                    Modify Seasons
                </button>
                <button
                    class="w-full rounded-md px-3 py-2 text-left text-sm text-rose-600 hover:bg-rose-50 dark:text-rose-300 dark:hover:bg-rose-500/10"
                    @click="emit('delete'); menuOpen = false"
                >
                    Delete
                </button>
            </div>
        </div>
    </nav>
</template>
