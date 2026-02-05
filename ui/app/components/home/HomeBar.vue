<script setup lang="ts">
import { navigateTo } from "#imports";
import { getTitle } from "~/utils/titles";
import CoverGet from "~/components/img/CoverGet.vue";

interface Item {
    ext: string;
    manga_id: string;
    number: number;
    status: "Dropped" | "Hiatus" | "Ongoing" | "Completed" | "Upcoming";
    tags: {
        description?: string | null | undefined;
        sex:
            | "Female"
            | "Male"
            | "Both"
            | "None"
            | "FemaleMale"
            | "MaleFemale"
            | "Unknown";
        tag: string;
    }[];
    titles: { [key: string]: string[] };
}

defineProps<{ route: string; title: string; noMore: boolean; data: Item[] }>();
</script>

<template>
    <div class="w-full flex flex-col">
        <div class="flex w-full items-center justify-between gap-3 px-2 sm:px-3">
            <div class="text-base font-semibold text-slate-900 dark:text-slate-100">
                {{ title }}
            </div>
            <div>
                <button
                    v-if="!noMore"
                    class="rounded-full border border-slate-200 bg-white px-3 py-1 text-xs font-semibold text-slate-700 shadow-sm transition hover:bg-slate-50 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200 dark:hover:bg-slate-800"
                    @click="() => navigateTo(route)"
                >
                    More
                </button>
            </div>
        </div>

        <div class="mt-3 rounded-lg bg-slate-100/70 p-2 dark:bg-slate-900/70">
            <div
                class="flex h-full snap-x snap-proximity gap-4 overflow-x-auto overflow-y-hidden rounded-md pb-1 pt-px pb-px hide-scroll scroll-pl-1 scroll-pr-1"
            >
                <div
                    v-for="item in data"
                    :key="item.manga_id"
                    class="group flex h-[17.5rem] flex-shrink-0 snap-start cursor-pointer flex-col overflow-hidden rounded-lg bg-white ring-1 ring-slate-200 transition hover:-translate-y-1 hover:shadow-lg sm:h-[19rem] dark:bg-slate-800 dark:ring-slate-700 first:ml-px last:mr-px"
                    style="aspect-ratio: 3/5"
                    @click="() => navigateTo(`/reader/${item.manga_id}`)"
                >
                    <CoverGet
                        :manga-id="item.manga_id"
                        :ext="item.ext"
                        :status="item.status"
                        class-list="h-[calc(100%-3rem)] w-full object-cover"
                    />
                    <div
                        class="flex h-[3rem] items-center justify-center px-2"
                        @click="() => navigateTo(`/info/${item.manga_id}`)"
                    >
                        <p
                            class="line-clamp-2 text-center text-sm font-semibold text-slate-900 dark:text-slate-100"
                        >
                            {{ getTitle(item.titles) }}
                        </p>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.hide-scroll::-webkit-scrollbar {
    display: none;
}

.hide-scroll {
    scrollbar-width: none;
}
</style>
