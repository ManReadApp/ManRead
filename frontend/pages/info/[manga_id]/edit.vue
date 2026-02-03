<script setup lang="ts">
import {getTitle} from "~/utils/titles";

const {$manRead} = useNuxtApp();
const route = useRoute()

const mangaId = Array.isArray(route.params.manga_id) ? route.params.manga_id[0] : route.params.manga_id;
const {data: value, error} = await useAsyncData(`info-data-${mangaId}`, async () => {
  const data = await $manRead("/api/v1/manga/detail/info", {
    method: "POST",
    body: {id: mangaId},
    headers: {Authorization: `Bearer ${await getAccessToken()}`}
  });
  let map: Record<string, string[]> = {};
  for (const tag of data.tags) {
    (map[tag.sex] ??= []).push(tag.tag);
  }
  let map2: Record<string, string[]> = {};
  for (const scraper of data.scrapers) {
    (map2[scraper.channel] ??= []).push(scraper.url);
  }
  return {
    manga_id: data.manga_id,
    titles: data.titles,
    status: data.status,
    kind: data.kind,
    description: data.description,
    sources: data.sources.map(v => v.url),
    tags: map,
    scrapers: map2,
    ppls: {
      "Artist": data.artists,
      "Author": data.authors,
      "Publisher": data.publishers,
    }
  }
})

useHead({
  title: (error.value ? 'Manga does not exist' : (value.value ? "Edit " + getTitle(value.value.titles) : "Edit Manga Loading")) + " - ManRead",
})
</script>
<template>
  <AddManga v-if="value?.manga_id" :initial-tags="value.tags" :initial-titles="value.titles"
            :manga-id="value.manga_id"
            :initial-status="value.status"
            :initial-kind="value.kind"
            :initial-description="value.description ?? ''" :initial-ppls="value.ppls" :initial-scrapers="value.scrapers"
            :initial-sources="value.sources"/>
</template>

<style scoped>

</style>