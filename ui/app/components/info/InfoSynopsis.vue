<script lang="ts" setup>
import SynopsisAllTags from "~/components/info/SynopsisAllTags.vue";

defineProps<{
    summary: string[];
    value: any;
}>();
</script>

<template>
    <div class="min-w-0" style="grid-area: synopsis">
        <div class="break-words">
            <div
                class="overflow-hidden transition-[max-height,height]"
                style="mask-image: linear-gradient(black 0%, black 0%)"
            >
                <div>
                    <div class="text-sm !py-0 text-slate-700 dark:text-slate-200">
                        <div class="md-md-container">
                            <p v-for="(chunk, i) in summary" :key="i">
                                {{ chunk }}
                            </p>
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
</template>
