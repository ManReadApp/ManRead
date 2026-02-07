<script lang="ts" setup>
import { getOtherTitles, getTitle } from "~/utils/titles";
import { getCoverUrl } from "~/utils/cover";
import { useWindowSize } from "@vueuse/core";
import InfoNav from "~/components/info/InfoNav.vue";
import InfoHero from "~/components/info/InfoHero.vue";
import InfoSynopsis from "~/components/info/InfoSynopsis.vue";
import InfoChapters from "~/components/info/InfoChapters.vue";

const { $manRead } = useNuxtApp();
const route = useRoute();

const mangaId = Array.isArray(route.params.manga_id)
    ? route.params.manga_id[0]
    : route.params.manga_id;
const { data: value, error } = await useAsyncData(
    `info-data-${mangaId}`,
    async () => {
        if (!mangaId) return null;
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
        class="info-root no-scrollbar h-screen w-full overflow-y-auto bg-white text-slate-900 dark:bg-[#191A1C] dark:text-slate-100"
    >
        <div style="height: var(--navbar-height)" />
        <InfoNav
            :is-favorite="value?.favorite"
            @back="$router.back()"
            @toggle-favorite="
                () => (value ? (value.favorite = !value.favorite) : null)
            "
            @edit="() => {}"
            @add-chapter="() => {}"
            @modify-seasons="() => {}"
            @delete="() => {}"
        />
        <div
            v-if="value && titles"
            class="layout-container manga has-gradient px-4"
        >
            <InfoHero
                :value="value"
                :manga-id="mangaId!"
                :titles="titles"
                :img-url="img_url"
                :creator-string="creator_string"
                :width="width"
            />
            <div style="grid-area: stats" />
            <InfoSynopsis :summary="summary" :value="value" />
            <InfoChapters
                :value="value"
                :chapters="chapters"
                :ascending="ascending"
                :manga-id="mangaId ?? ''"
                :width="width"
                @toggle-sort="() => (ascending = !ascending)"
            />
        </div>
    </div>
</template>

<style scoped>
.info-root {
    --banner-height: 13.5rem;
    --md-accent-10: 226 232 240;
    --md-accent: 203 213 225;
    --md-background1: 255 255 255;

    --md-primary: 67 45 215;
    --md-accent-20-hover: 148 163 184;
    --md-status-green: 34 197 94;
    --navbar-height: 2rem;
    --main-color: 67 45 215;
}
:global(html.dark) {
    --md-background: 25 26 28;
}

:global(.dark) .info-root {
    --md-accent-10: 30 41 59;
    --md-accent: 15 23 42;
    --md-background: 25 26 28;
    --md-primary: 129 140 248;
    --md-accent-20-hover: 51 65 85;
    --md-status-green: 34 197 94;
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
    background-color: rgb(var(--md-accent-10));
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
