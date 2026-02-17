<script lang="ts" setup>
import { getOtherTitles, getTitle } from "~/utils/titles";
import { getCoverUrl } from "~/utils/cover";
import { useWindowSize } from "@vueuse/core";
import CoverGet from "~/components/img/CoverGet.vue";
import Flag from "~/components/img/Flag.vue";
import SynopsisSegment from "~/components/info/synopsis-segment.vue";
import SynopsisAllTags from "~/components/info/SynopsisAllTags.vue";

const { $manRead } = useNuxtApp();
const route = useRoute();

const mangaId = Array.isArray(route.params.manga_id)
    ? route.params.manga_id[0]
    : route.params.manga_id;
const { data: value, error } = await useAsyncData(
    `info-data-${mangaId}`,
    async () => {
        return await $manRead("/api/v1/manga/detail/info", {
            method: "POST",
            body: { id: mangaId },
            headers: { Authorization: `Bearer ${await getAccessToken()}` },
        });
    },
);

useHead({
    title:
        (error.value
            ? "Manga does not exist"
            : value.value
              ? getTitle(value.value.titles)
              : "Manga Details Loading") + " - ManRead",
});
//TODO: add relations

const img_url = computed(() => {
    return value.value
        ? getCoverUrl(value.value.manga_id, value.value.cover_ext[0] ?? "")
        : "";
});

const chapter_link = (chapter_id: string) => {
    return `/reader/${mangaId}/${chapter_id}`;
};

const titles = computed(() => {
    if (!value.value) return null;
    const title = getTitle(value.value.titles);
    return {
        title: title,
        other: getOtherTitles(value.value.titles, title),
    };
});

const summary = computed(() => {
    if (!value.value || !value.value.description) return [];
    return value.value.description.split("\n");
});

const ascending = ref(true);
const toggle = () => {
    ascending.value = !ascending.value;
};

const chapters = computed(() => {
    if (!value.value) return [];
    if (ascending.value) {
        return value.value.chapters;
    } else {
        return [...value.value.chapters].reverse();
    }
});

const creator_string = computed(() => {
    if (!value.value) return "";
    return [
        ...new Set([
            ...value.value.authors,
            ...value.value.artists,
            value.value.uploader,
        ]),
    ]
        .sort()
        .join(", ");
});

const { width } = useWindowSize();
</script>

<template>
    <div
        class="bg-white dark:bg-[#191A1C] w-full h-full overflow-auto"
        style="
            --md-background: 25 26 28;
            --navbar-height: 2rem;
            --main-color: 67 45 215;
        "
    >
        <!--<div class="dark bg-white dark:bg-[#191A1C] w-full h-full" style="--md-background: 25 26 28; --navbar-height: 2rem;">-->
        <div style="height: var(--navbar-height)" />
        <nav
            class="fixed top-2 left-0 w-full flex items-center justify-between p-4 bg-transparent z-10 h-0"
        >
            <button
                @click="$router.back()"
                class="text-white -translate-x-3 -translate-y-2 cursor-pointer"
            >
                <LucideChevronLeft />
            </button>

            <div class="flex gap-4">
                <button
                    class="text-red-500 cursor-pointer"
                    @click="
                        () =>
                            value ? (value.favorite = !value.favorite) : null
                    "
                >
                    <LucideHeart :style="value?.favorite ? 'fill: red' : ''" />
                </button>
                <NuxtLink to="https://google.com/profile" class="text-white">
                    <LucideEllipsisVertical />
                </NuxtLink>
            </div>
        </nav>
        <div
            v-if="value && titles"
            class="layout-container manga has-gradient px-4"
        >
            <div
                v-if="width >= 640"
                class="absolute top-0 left-0 z-0 w-full h-[640px] blur-xl"
                :style="`background: radial-gradient(circle at top, rgb(var(--md-background) / 0.8), rgb(var(--md-background)) 75%), no-repeat top 35% center / 100% url(${img_url});`"
            />
            <div class="banner-container z-0 top-0">
                <div
                    class="banner-image"
                    :style="`background-image: url('${img_url}'); width: 100%;`"
                />
                <div class="banner-shade" />
            </div>
            <div style="grid-area: art">
                <div>
                    <a
                        class="group flex items-start relative mb-auto select-none"
                        :href="img_url"
                        target="_self"
                    >
                        <div
                            class="flex opacity-0 group-hover:opacity-70 transition-opacity items-center justify-center inset-0 absolute bg-black bg-opacity-50 pointer-events-none rounded z-1"
                        >
                            <svg
                                class="icon xLarge text-white"
                                fill="none"
                                height="24"
                                viewBox="0 0 24 24"
                                width="24"
                                xmlns="http://www.w3.org/2000/svg"
                            >
                                <path
                                    d="m9.5 13.09 1.41 1.41-4.5 4.5H10v2H3v-7h2v3.59zm1.41-3.59L9.5 10.91 5 6.41V10H3V3h7v2H6.41zm3.59 3.59 4.5 4.5V14h2v7h-7v-2h3.59l-4.5-4.5zM13.09 9.5l4.5-4.5H14V3h7v7h-2V6.41l-4.5 4.5z"
                                    fill="currentColor"
                                ></path>
                            </svg>
                        </div>
                        <CoverGet
                            :status="value.status"
                            class-list="rounded shadow-md w-full h-auto"
                            :manga_id="value.manga_id"
                            :ext="value.cover_ext[0].value ?? ''"
                        />
                    </a>
                </div>
            </div>
            <div class="title text-white">
                <p
                    class="mb-1"
                    style="
                        line-height: 1.1em;
                        overflow-wrap: break-word;
                        text-shadow: rgba(0, 0, 0, 0.3) 1px 2px 4px;
                        font-size: 2rem;
                        width: 800px;
                    "
                >
                    {{ titles.title }}
                </p>
                <div
                    class="font-normal line-clamp-2 text-base sm:text-xl inline-block leading-5"
                    title="{{titles.other}}"
                >
                    {{ titles.other }}
                </div>
                <div class="grow hidden sm:block"></div>
                <div class="flex flex-row gap-2">
                    <div class="font-normal text-xs sm:text-base truncate">
                        {{ creator_string }}
                    </div>
                </div>
            </div>

            <div class="sm:ml-2 relative" style="grid-area: buttons">
                <button
                    class="bg-indigo-700 text-white glow px-6 py-3 justify-center content-center h-auto rounded cursor-pointer font-semibold"
                >
                    {{ value.progress ? "Continue" : "Start" }} Reading
                </button>
            </div>
            <div style="grid-area: stats" />
            <div class="sm:mx-2 z-1" style="grid-area: info">
                <div class="flex gap-1 flex-wrap items-center">
                    <div
                        class="flex flex-wrap gap-1 tags-row"
                        style="max-height: calc(1em + 0rem)"
                    >
                        <a
                            v-for="tag in value?.tags"
                            class="tag bg-accent"
                            href="https://mangadex.com/tag/391b0423-d847-456f-aff0-8b0cfc03066b/action"
                            >{{ tag.tag }}</a
                        >
                    </div>
                </div>
            </div>
            <div class="min-w-0" style="grid-area: synopsis">
                <div style="word-break: break-word">
                    <div
                        class="overflow-hidden transition-[max-height,height]"
                        style="mask-image: linear-gradient(black 0%, black 0%)"
                    >
                        <div>
                            <div class="text-sm !py-0">
                                <div class="md-md-container">
                                    <p v-for="chunk in summary">{{ chunk }}</p>
                                </div>
                            </div>

                            <SynopsisAllTags
                                class="xl:hidden"
                                :titles="value.titles"
                                :sources="value.sources"
                                :tags="value.tags"
                                :artists="value.artists"
                                :authors="value.authors"
                                :publishers="value.publishers"
                            />
                        </div>
                    </div>
                </div>
            </div>
            <div class="mb-8 z-10" style="grid-area: content">
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
                        <button @click="() => (ascending = !ascending)">
                            {{ ascending ? "Ascending" : "Descending" }}
                        </button>
                        <ul class="chapter-list">
                            <li
                                @click="
                                    () => navigateTo(chapter_link(chapter.id))
                                "
                                v-for="chapter in chapters"
                                :key="chapter.id"
                            >
                                {{ chapter.chapter }}
                                {{ chapter.titles[0] ?? "" }}
                            </li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
:root {
    --banner-height: 13.5rem;
    --md-accent-10: red;
    --md-accent: red;
    --md-background: 25 26 28;
    --md-primary: red;
    --md-accent-20-hover: red;
    --md-status-green: green;
}

.md-md-container.dense h1,
.md-md-container.dense h2,
.md-md-container.dense h3,
.md-md-container.dense h4,
.md-md-container.dense h5,
.md-md-container.dense h6 {
    font-size: 1em;
}

.md-md-container.noEmptyLines p {
    margin-bottom: 0;
    margin-top: 0;
}

.md-md-container > :first-child {
    margin-top: 0;
}

.md-md-container > :last-child {
    margin-bottom: 0;
}

.md-md-container pre {
    display: block;
    font-size: 13px;
    line-height: 1.42857143;
    margin: 0 0 10px;
    word-break: break-all;
    word-wrap: break-word;
    background-color: rgb(var(--md-accent-10));
    border-radius: 0.25rem;
    box-shadow: inset 0 0 10px #0000001a;
    padding: 1rem;
}

.chapter-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 16px; /* Adjusts space between items */
}

.chapter-list > li {
    align-items: center;
    border: 1px solid rgba(0, 0, 0, 0.12);
    border-radius: 3px;
    box-sizing: border-box;
    color: rgba(0, 0, 0, 0.87);
    display: flex;
    font-size: 14px;
    height: 42px;
    line-height: 1.5;
    padding: 0 12px;
    position: relative;
    cursor: pointer;
    text-align: left;
}

.md-md-container pre > code {
    border-radius: revert;
    font-size: revert;
    padding: revert;
}

.md-md-container code {
    background-color: rgb(var(--md-accent-10));
    border-radius: 0.125rem;
    font-size: 90%;
    padding: 2px 4px;
}

.md-md-container img {
    max-width: 35%;
}

.md-md-container hr {
    color: inherit;
    margin-bottom: 1rem;
    margin-top: 1rem;
    opacity: 0.1;
}

.md-md-container a {
    color: rgb(var(--md-primary));
}

.md-md-container,
.md-md-container * {
    all: revert;
}

.md-md-container table {
    border-collapse: collapse;
    border: 2px solid rgb(var(--md-accent-10));
    border-radius: 0.25rem;
    display: block;
    max-width: fit-content;
    overflow-x: auto;
    word-break: normal;
}

.md-md-container td,
.md-md-container th {
    padding: 0.25rem 0.5rem;
    text-align: center;
}

.md-md-container thead {
    border-bottom: 2px solid;
    border-color: rgb(var(--md-accent-10));
}

.md-md-container tr {
    border-bottom: 1px solid;
    border-color: rgb(var(--md-accent-10));
}

.md-md-container tr:nth-child(2n) {
    background-color: rgb(var(--md-accent));
}

.md-md-container tr:nth-child(odd) {
    background-color: rgb(var(--md-background) / 0.6);
}

.controls > svg {
    align-items: center;
    display: flex;
    justify-content: center;
    margin: 0.625rem;
}

.arrow > * {
    filter: drop-shadow(0 0 2px #000);
}

img {
    -o-object-fit: cover;
    object-fit: cover;
    -o-object-position: center center;
    object-position: center center;
}

.aspect img {
    height: 100%;
    top: 0;
}

.tag svg:not(:last-child) {
    margin-right: 0.5rem;
}

.tag.dot svg {
    margin: -0.3125rem -0.125rem -0.3125rem -0.25rem;
}

.tag.dot.content svg,
.tag.dot.small svg {
    margin: -0.5rem -0.25rem -0.5rem -0.5rem;
}

.tag {
    border-radius: 0.25rem;
    display: inline-block;
    font-size: 0.625rem;
    font-weight: 700;
    line-height: 1.5em;
    margin-bottom: auto;
    margin-top: auto;
    padding: 0 0.375rem;
    text-transform: uppercase;
}

.glow {
    box-shadow: 0 0 24px -8px rgb(var(--main-color));
}

.manga-card-dense .manga-card .title,
.manga-card.dense .title {
    height: 1.5em;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.manga-card-cover-only .manga-card .title,
.manga-card.cover-only .title {
    background: linear-gradient(180deg, transparent, rgba(0, 0, 0, 0.8));
    bottom: 0;
    display: -webkit-box;
    left: 0;
    margin-right: 0 !important;
    overflow: hidden;
    padding: 1rem 0.5rem 0.25rem;
    position: absolute;
    text-shadow: 0 0 3px #000;
    width: 100%;
    z-index: 1;
    -webkit-box-orient: vertical;
}

@media (min-width: 40rem) {
    .manga-card-cover-only .manga-card .title,
    .manga-card.cover-only .title {
        display: -webkit-box;
        overflow: hidden;
        -webkit-box-orient: vertical;
    }
}

@media (min-width: 48rem) {
    .manga-card-cover-only .manga-card .title,
    .manga-card.cover-only .title {
        display: -webkit-box;
        overflow: hidden;
        -webkit-box-orient: vertical;
    }
}

@media (min-width: 64rem) {
    .manga-card-cover-only .manga-card .title,
    .manga-card.cover-only .title {
        display: -webkit-box;
        overflow: hidden;
        -webkit-box-orient: vertical;
    }
}

.manga-card-cover-only .manga-card .title > span,
.manga-card.cover-only .title > span {
    flex-grow: 1;
    font-size: 0.875rem;
    font-weight: 400;
    overflow: hidden;
    --tw-text-opacity: 1;
}

@media (min-width: 40rem) {
    .manga-card-cover-only .manga-card .title,
    .manga-card.cover-only .title {
        font-size: 1rem;
        line-height: 1.5rem;
        padding: 1rem 0.5rem 0.5rem;
    }
}

.manga-card-cover-only .manga-card .title,
.manga-card.cover-only .title {
    transition-duration: 75ms;
    transition-property: padding;
    transition-timing-function: ease-in-out;
}

.manga-card-cover-only .manga-card .cover img,
.manga-card.cover-only .cover img {
    height: 100%;
    left: 0;
    position: absolute;
    top: 0;
    transition-duration: 75ms;
    transition-property: width, height;
    transition-timing-function: ease-in-out;
    width: 100%;
}

@media (hover: hover) {
    .manga-card-cover-only .manga-card:hover .cover img {
        height: 102%;
        width: 102%;
    }

    .manga-card-cover-only .manga-card:hover .title {
        padding: 100% 0.5rem 1rem;
    }
}

.md-inputwrap .md-label > span {
    color: rgb(var(--md-primary));
}

.md-inputwrap input {
    background: none;
    position: relative;
    width: 100%;
    z-index: 1;
}

.md-inputwrap input::-moz-placeholder {
    opacity: 0.6;
}

.md-inputwrap input::placeholder {
    opacity: 0.6;
}

.md-inputwrap input:focus {
    outline: none;
}

.md-disabled input {
    color: inherit;
    opacity: 0.2;
}

.md-has-error i {
    color: inherit !important;
}

.layout-container {
    --banner-overlap: var(--navbar-height);
    --banner-top: calc(var(--banner-overlap) * -1);
    display: grid;
    position: relative;
}

.layout-container.manga {
    --banner-overlay-gradient: linear-gradient(
        to bottom,
        rgb(var(--md-background)/0.8) 0%,
        rgb(var(--md-background)) 100%
    );
    gap: 0.75rem 1rem;
    grid-template-areas:
        "art title"
        "art stats"
        "info info"
        "buttons buttons"
        "synopsis synopsis"
        "content content";
    grid-template-columns: 100px auto;
}

@media (min-width: 40rem) {
    .layout-container.manga {
        --banner-filter: blur(4px);
        --banner-overlay-gradient: linear-gradient(
            67.81deg,
            rgba(0, 0, 0, 0.64) 35.51%,
            transparent
        );
        gap: 1rem;
        grid-template-areas: "left art title right" "left art buttons right" "left art info right" "left art stats right" "left art padding right" "left synopsis synopsis right" "left content content right";
        grid-template-columns: 1fr 200px minmax(0, calc(1240px - 3.5rem)) 1fr;
    }
}

@media not (min-width: 40rem) {
    .layout-container.has-gradient:not(.manga) {
        --banner-overlay-gradient: linear-gradient(
            to bottom,
            rgb(var(--md-background)) 0%,
            rgb(var(--md-background)/0.8) 56px,
            rgb(var(--md-background)/0) 50%
        );
    }
}

.banner-container {
    clip: rect(0, auto, auto, 0);
    clip-path: inset(0 0);
    left: 0;
    position: absolute;
    right: 0;
    top: var(--banner-top);
    width: auto;
}

.banner-container,
.banner-image {
    height: calc(13.5rem + var(--navbar-height));
}

.banner-image {
    background-position: center 25%;
    background-size: cover;
    position: fixed;
    transition: width 0.15s ease-in-out;
    width: 100%;
}

.bg-accent {
    background-color: rgb(240, 241, 242);
}

.banner-container > .banner-shade {
    -webkit-backdrop-filter: var(--banner-filter);
    backdrop-filter: var(--banner-filter);
    background: var(--banner-overlay-gradient);
    bottom: 0;
    height: auto;
    left: 0;
    pointer-events: none;
    position: absolute;
    right: 0;
    top: 0;
    width: auto;
}

.title {
    display: flex;
    flex-direction: column;
    font-size: 24px;
    font-weight: 700;
    grid-area: title;
    line-height: 100%;
    min-width: 0;
    position: relative;
}

@media (min-width: 40rem) {
    .title {
        padding-bottom: 0.5rem;
        padding-left: 0.5rem;
        padding-right: 0.5rem;
        --tw-text-opacity: 1;
        height: calc(13.5rem);
        justify-content: flex-end;
    }
}
</style>
