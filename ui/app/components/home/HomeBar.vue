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
    <div class="w-full h-80 flex flex-col m-2 my-8">
        <div class="w-full flex justify-between space-x-2 px-3 py-1">
            <div class="font-semibold">
                {{ title }}
            </div>
            <div>
                <button
                    v-if="!noMore"
                    class="px-1 py-0.5 border border-gray-200 shadow bg-white hover:bg-gray-100 text-gray-800 font-semibold text-xs rounded"
                    @click="() => navigateTo(route)"
                >
                    More
                </button>
            </div>
        </div>

        <div class="bg-gray-100 rounded-md w-full h-full p-3">
            <div
                class="flex overflow-x-auto h-full space-x-2 hide-scroll overflow-hidden rounded-md"
            >
                <div
                    v-for="item in data"
                    :key="item.manga_id"
                    class="bg-[#EDEDED] h-full flex-shrink-0 cursor-pointer rounded-md overflow-hidden"
                    style="aspect-ratio: 3/5"
                    @click="() => navigateTo(`/reader/${item.manga_id}`)"
                >
                    <CoverGet
                        :manga-id="item.manga_id"
                        :ext="item.ext"
                        :status="item.status"
                        class-list="w-full h-[calc(100%-3rem)] object-cover"
                    />
                    <div
                        class="flex items-center justify-center h-[3rem] mx-1 cursor-pointer"
                        @click="() => navigateTo(`/info/${item.manga_id}`)"
                    >
                        <p
                            class="text-center text-gray-900 line-clamp-2 text-sm font-semibold text-ellipsis"
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
