<script lang="ts" setup>
import CoverGet from "~/components/img/CoverGet.vue";

defineProps<{
    value: any;
    mangaId: string;
    titles: { title: string; other?: string | null };
    imgUrl: string;
    creatorString: string;
    width: number;
}>();
</script>

<template>
    <div
        v-if="width >= 640"
        class="absolute left-0 top-0 z-0 h-[640px] w-full blur-xl"
        :style="`background: radial-gradient(circle at top, rgb(var(--md-background) / 0.8), rgb(var(--md-background)) 75%), no-repeat top 35% center / 100% url(${imgUrl});`"
    />
    <div class="banner-container z-0 top-0">
        <div
            class="banner-image"
            :style="`background-image: url('${imgUrl}'); width: 100%;`"
        />
        <div class="banner-shade" />
    </div>

    <div style="grid-area: art">
        <div>
            <a
                class="group relative mb-auto flex select-none items-start"
                :href="imgUrl"
                target="_self"
            >
                <div
                    class="z-1 pointer-events-none absolute inset-0 flex items-center justify-center rounded bg-black/50 opacity-0 transition-opacity group-hover:opacity-70"
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
                        />
                    </svg>
                </div>
                <CoverGet
                    :status="value.status"
                    class-list="w-full h-auto rounded shadow-md"
                    :manga-id="value.manga_id"
                    :ext="value.cover_ext[0] ?? ''"
                />
            </a>
        </div>
    </div>

    <div class="title text-slate-900 dark:text-white sm:text-white">
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
            class="mt-2 inline-block line-clamp-2 text-sm font-normal leading-5 text-slate-700 sm:text-xl sm:text-slate-200"
            :title="titles.other ?? ''"
        >
            {{ titles.other }}
        </div>
        <div class="hidden grow sm:block" />
        <div class="mt-2 flex flex-row gap-2">
            <div
                class="truncate text-xs font-normal text-slate-700 sm:text-base sm:text-slate-200"
            >
                {{ creatorString }}
            </div>
        </div>
    </div>

    <div class="relative sm:ml-2" style="grid-area: buttons">
        <NuxtLink :to="`/reader/${mangaId}/`"
            ><button
                class="glow h-auto cursor-pointer content-center justify-center rounded bg-indigo-700 px-6 py-3 font-semibold text-white"
            >
                {{ value.progress ? "Continue" : "Start" }} Reading
            </button></NuxtLink
        >
    </div>

    <div class="sm:mx-2 z-1" style="grid-area: info">
        <div class="flex flex-wrap items-center gap-1">
            <div
                class="tags-row flex flex-wrap gap-1"
                style="max-height: calc(1em + 0rem)"
            >
                <a
                    v-for="(tag, i) in value?.tags"
                    :key="i"
                    class="tag bg-accent"
                    href="https://mangadex.com/tag/391b0423-d847-456f-aff0-8b0cfc03066b/action"
                    >{{ tag.tag }}</a
                >
            </div>
        </div>
    </div>
</template>

<style scoped>
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

.banner-container > .banner-shade {
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
        height: calc(13.5rem);
        justify-content: flex-end;
    }
}

.bg-accent {
    background-color: rgb(var(--md-accent-10));
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
</style>
