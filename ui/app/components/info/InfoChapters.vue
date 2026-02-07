<script lang="ts" setup>
import SynopsisAllTags from "~/components/info/SynopsisAllTags.vue";
import { navigateTo } from "#imports";

defineProps<{
    value: any;
    chapters: any[];
    ascending: boolean;
    mangaId: string;
    width: number;
}>();

const emit = defineEmits(["toggleSort"]);

const chapterLink = (mangaId: string, chapterId: string) =>
    `/reader/${mangaId}/${chapterId}`;
</script>

<template>
    <div class="z-10 mb-8" style="grid-area: content">
        <div class="flex">
            <div v-if="width >= 1280" class="flex-1/4">
                <SynopsisAllTags
                    :titles="value.titles"
                    :sources="value.sources"
                    :tags="value.tags"
                    :artists="value.artists"
                    :authors="value.authors"
                    :publishers="value.publishers"
                />
            </div>
            <div :class="width >= 1280 ? 'flex-3/4' : 'flex-1'">
                <div class="flex items-center justify-between">
                    <h3 class="text-base font-semibold text-slate-900 dark:text-slate-100">
                        Chapters
                    </h3>
                    <button
                        class="text-sm font-medium text-indigo-600 hover:text-indigo-500 dark:text-indigo-300 dark:hover:text-indigo-200"
                        @click="emit('toggleSort')"
                    >
                        {{ ascending ? "Ascending" : "Descending" }}
                    </button>
                </div>
                <ul
                    class="mt-4 grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-3"
                >
                    <li
                        v-for="chapter in chapters"
                        :key="chapter.id"
                        class="flex h-12 cursor-pointer items-center rounded-md border border-slate-200 bg-white/80 px-3 text-sm text-slate-800 transition hover:border-slate-300 hover:bg-slate-50/80 dark:border-slate-700 dark:bg-slate-900/80 dark:text-slate-100 dark:hover:border-slate-500 dark:hover:bg-slate-800/60"
                        @click="navigateTo(chapterLink(mangaId, chapter.id))"
                    >
                        <span class="truncate">
                            {{ chapter.chapter }} {{ chapter.titles[0] ?? "" }}
                        </span>
                    </li>
                </ul>
            </div>
        </div>
    </div>
</template>
