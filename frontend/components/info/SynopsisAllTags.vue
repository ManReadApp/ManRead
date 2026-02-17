<script setup lang="ts">
import SynopsisSegment from "~/components/info/synopsis-segment.vue";
import Flag from "~/components/img/Flag.vue";

defineProps<{ authors: string[], artists: string[], publishers: string[], tags: any[], sources: any[], titles: Record<string, {items: string[]}> }>();
const extract_hostname = (url: string) => {
  const hostname = new URL(url).hostname;
  const parts = hostname.split('.');

  if (parts[0] === 'www') {
    parts.shift();
  }

  parts.pop();

  return parts.join('.');
};
</script>

<template>
  <div class="flex flex-wrap gap-x-4 gap-y-2">
    <div class="mb-2" v-if="authors.length">
      <SynopsisSegment label="Author" :items="authors.map(v => {return {label: v, href:'#'}})"/>
    </div>
    <div class="mb-2" v-if="artists.length">
      <SynopsisSegment label="Artist" :items="artists.map(v => {return {label: v, href:'#'}})"/>
    </div>
    <div class="mb-2" v-if="publishers.length">
      <SynopsisSegment label="Artist" :items="publishers.map(v => {return {label: v, href:'#'}})"/>
    </div>
    <div class="mb-2" v-if="tags.length">
      <SynopsisSegment label="Tags" :items="tags.map(v => {return {label: `${v.sex} ${v.tag}`, href:'#'}})"/>
    </div>
    <div class="mb-2" v-if="sources.length">
      <SynopsisSegment label="Links"
                       :items="sources.map(v => {return {label: v.icon_uri || extract_hostname(v.url), href:v.url}})"/>
    </div>

    <div class="w-full">
      <div class="font-bold mb-1">Alternative Titles</div>
      <div v-for="(titles, lang) in titles">
        <div class="mb-1 flex gap-x-2 alt-title" v-for="title in titles.items">
          <Flag :lang="lang as string" :title="title"/>
          <span>{{ title }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>

</style>