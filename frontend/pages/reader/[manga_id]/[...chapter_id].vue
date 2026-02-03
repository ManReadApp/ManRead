<script setup lang="ts">
import MainReader from "~/components/reader/MainReader.vue";
import {getTitle} from "~/utils/titles";

const {$manRead} = useNuxtApp()
const route = useRoute()
const router = useRouter();


async function getdata() {
  const mangaId = String(route.params.manga_id)

  const chapterId = route.params.chapter_id ? route.params.chapter_id[0] : null;
  const data = await $manRead('/api/v1/manga/reader', {
    method: "POST", body: {
      manga_id: mangaId,
      chapter_id: chapterId,
    },
    headers: {Authorization: `Bearer ${await getAccessToken()}`}
  });

  if (chapterId === null || route.params.chapter_id.length === 1) {
    const ch = data.chapters.find((ch) => ch.chapter_id === data.open_chapter);
    await router.replace({params: {manga_id: mangaId, chapter_id: [data.open_chapter, ch?.chapter ?? 'unknown']}})
  }
  return data;
}

const {data: value, error} = await useAsyncData("reader-data", getdata);
useHead({
  title: error.value ? "Reader failed to load - ManRead" : value.value ? `Read ${getTitle(value.value.titles)} - ManRead` : "Loading Reader - ManRead",
});

</script>
<template>
  <AuthGuard :roles="[]">
    <ClientOnly>
      <MainReader v-if="value" :value="value"/>
    </ClientOnly>
  </AuthGuard>
</template>