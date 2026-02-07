<script setup lang="ts">
import SynopsisSegment from "~/components/info/SynopsisSegment.vue";
import Flag from "~/components/img/Flag.vue";

type Tag = {
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
};
defineProps<{
    authors: string[];
    artists: string[];
    publishers: string[];
    tags: Tag[];
    sources: {
        icon_uri: string;
        url: string;
    }[];
    titles: Record<string, string[]>;
}>();
const extract_hostname = (url: string) => {
    const hostname = new URL(url).hostname;
    const parts = hostname.split(".");

    if (parts[0] === "www") {
        parts.shift();
    }

    parts.pop();

    return parts.join(".");
};
</script>

<template>
    <div class="flex flex-wrap gap-x-4 gap-y-2">
        <div v-if="authors.length" class="mb-2">
            <SynopsisSegment
                label="Author"
                :items="
                    authors.map((v) => {
                        return { label: v, href: '#' };
                    })
                "
            />
        </div>
        <div v-if="artists.length" class="mb-2">
            <SynopsisSegment
                label="Artist"
                :items="
                    artists.map((v) => {
                        return { label: v, href: '#' };
                    })
                "
            />
        </div>
        <div v-if="publishers.length" class="mb-2">
            <SynopsisSegment
                label="Artist"
                :items="
                    publishers.map((v) => {
                        return { label: v, href: '#' };
                    })
                "
            />
        </div>
        <div v-if="tags.length" class="mb-2">
            <SynopsisSegment
                label="Tags"
                :items="
                    tags.map((v) => {
                        return { label: `${v.sex} ${v.tag}`, href: '#' };
                    })
                "
            />
        </div>
        <div v-if="sources.length" class="mb-2">
            <SynopsisSegment
                label="Links"
                :items="
                    sources.map((v) => {
                        return {
                            label: v.icon_uri || extract_hostname(v.url),
                            href: v.url,
                        };
                    })
                "
            />
        </div>

        <div class="w-full">
            <div class="font-bold mb-1">Alternative Titles</div>
            <div v-for="(titles_, lang, i) in titles" :key="i">
                <div
                    v-for="(title, j) in titles_"
                    :key="j"
                    class="mb-1 flex gap-x-2 alt-title"
                >
                    <Flag :lang="lang as string" :title="title" />
                    <span>{{ title }}</span>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped></style>
