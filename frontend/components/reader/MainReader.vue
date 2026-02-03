<template>
  <div class="w-full h-full">
    <div class="w-full h-full" ref="divRef" :style="containerStyle">
      <SingleReader v-if="mode === 's'" :items="render_images" :cache="cache"
                    :left-to-right="direction === 'l2r'" @jump-to-page="(p) => setProgress(p, false, true)"/>
      <DoubleReader v-else-if="mode === 'd'" :items="render_images" :cache="cache"
                    @jump-to-page="(p) => setProgress(p, false, true)"
                    :left-to-right="direction === 'l2r'"/>
      <VerticalReader r v-else-if="mode === 'v'" :images="render_images" :cache="cache" :direction="direction" :mode="mode"/>
    </div>
    <DirectionIndicator :double="mode == 'd' && page[0] < page[1]" :direction="mode === 'v' ? 't2b' : direction"
                        :page="page[0]" :right="true" :navbar="navbar"/>

    <div v-if="!navbar" class="bg-orange-400 drop-shadow-2xl h-1 absolute bottom-0 left-0"
         :style="`width: ${info.progress*100}%`"></div>
    <ScrollOverlay v-if="mode === 'v' || mode=== 'h'"
                   @scroll="setProgressScroll" @click="() => navbar = !navbar"/>
    <TopNavbar v-if="navbar" :titles="value.titles" :manga_id="value.manga_id" :chapter="active_chapter ?? undefined"/>
    <BottomNavbar v-if="navbar" :progress="value.progress*100" :scrollbottom="scrollbar==='b'"
                  :chapters="info.chapters" v-model:reader-mode="processedMode" v-model:read-direction="direction"
                  :imageSettings="imageSettings" v-model:size-mode="sizeMode"
                  @clicked:open-sizing="() => {resize=true;navbar=false}"/>
    <Resizer v-if="resize"
             @click="() => resize=false"
             :input-horizontal="max_width"
             :input-vertical="max_height"
             :mode="sizeMode"
             @max-width="(msg) =>max_width = Math.round(msg)" @max-height="(msg) => max_height = Math.round(msg)"
             @reduce-width="(msg) => reduce_width = Math.round(msg)"
             @reduce-height="(msg) => reduce_height = Math.round(msg)"
             @percentage-width="(msg) => percentage_width = Math.round(msg * 100) / 100"
             @percentage-height="(msg) => percentage_height = Math.round(msg * 100) / 100"
    />
  </div>
</template>
<script lang="ts">
import {type Cache, type DataObject, getImages, type ImageItem, isDataObject, type Page} from '~/utils/img'
import SingleReader from "~/components/reader/SingleReader.vue";
import DoubleReader from "~/components/reader/DoubleReader.vue";
import {useElementSize} from "@vueuse/core";
import VerticalReader from "~/components/reader/VerticalReader.vue";
import Decimal from 'decimal.js';
import ScrollOverlay from "~/components/reader/ScrollOverlay.vue";
import TopNavbar from "~/components/reader/nav/TopNavbar.vue";
import BottomNavbar from "~/components/reader/nav/BottomNavbar.vue";
import DirectionIndicator from "~/components/reader/DirectionIndicator.vue";
import {reactive} from "vue";

type Ret = DataObject | "Unreachable" | "Unreachable2" | "Loading" | {
  offset: number,
  message: "Unreachable" | "Unreachable2" | "Loading"
};

type Tuple = [number, Ret[], boolean];
type Manga = {
  favorite: boolean,
  manga_id: string,
  kind: string,
  open_chapter: string,
  progress: number,
  description?: string | null | undefined,
  titles: any,
  chapters: {
    chapter: number,
    chapter_id: string,
    release_date?: string | null | undefined,
    sources: string[],
    titles: string[],
    versions: { [p: string]: string }
  }[],
};
//TODO: zoom
export default {
  components: {DirectionIndicator, BottomNavbar, TopNavbar, VerticalReader, DoubleReader, SingleReader, ScrollOverlay},
  mounted() {
    this.reload()
  },
  unmounted() {
    for (const item in this.cache) {
      const a = this.cache[item]
      if (a.value.success) {
        URL.revokeObjectURL(a.value.success)
      }
    }
  },
  props: {
    value: {
      type: Object as () => Manga,
      required: true,
    }
  },
  async setup(props) {
    type ImgInfoMap = Record<string, {
      link?: string | null | undefined;
      pages: Record<string, Page>;
    } | "loading">;
    const cache: Cache = {}
    const divRef = ref(null);

    const {width, height} = useElementSize(divRef);
    return {
      max_width: ref(99999999),
      max_height: ref(9999999),
      reduce_width: ref(0),
      reduce_height: ref(0),
      percentage_width: ref(100),
      percentage_height: ref(100),
      info: props.value,
      img_info_map: ref<ImgInfoMap>({}),
      render_images: ref<(Ret)[]>([]),
      cache: cache,
      mode: ref<'s' | 'd' | 'v' | 'h'>('v'),
      direction: ref<'l2r' | 'r2l'>('l2r'),
      page: ref([1, 0]),
      divRef,
      width, height,
      scrollbar: ref<'b' | 'l' | 'r'>('b'),
      navbar: ref(true),
      sizeMode: ref<'l' | '%' | 'm'>('l'),
      imageSettings: reactive({
        Blur: 0,
        Brightness: 100,
        Contrast: 100,
        Grayscale: 0,
        'Hue-rotate': 0,
        Invert: 0,
        Saturate: 100,
        Sepia: 0,
        Denoise: 0
      }),
      resize: ref(false)
    }
  },
  methods: {
    reload() {
      switch (this.mode) {
        case 's':
          this.getSinglePage();
          break;
        case 'd':
          this.getDoublePage();
          break;
        case 'v':
          this.getTDScroll();
          break;
        case 'h':
          this.getLRScroll();
          break;
      }
    },
    getTDScroll() {
      const a = this.getScroll(3, 4, false);
      this.start_download([...a[0], ...a[1], ...a[2]])
      this.render_images = a[0]
    },
    getScroll(negative: number, positive: number, horizontal: boolean): [Ret[], Ret[], Ret[]] {
      if (this.width <= 0 || this.height <= 0) return [[], [], []]
      const root_ = this.getRoot(horizontal, false);
      if (typeof root_ === 'string') {
        return [[{offset: 0, message: root_}], [], []]
      }
      const root = root_ as DataObject;
      const start = -root.offset * (horizontal ? this.calculateWidth(root.complete) : this.calculateHeight(root.complete));
      root.offset = start;
      let computed = start + (horizontal ? this.calculateWidth(root.complete) : this.calculateHeight(root.complete));
      let end = positive * (horizontal ? this.width : this.height);
      const pos: Ret[] = [root]
      const neg = [pos[0]]
      let render: Ret[] = [...pos];
      while (computed < end) {
        const item = pos[pos.length - 1];
        if (item == "Unreachable" || item == "Unreachable2" || item == "Loading") {
          break;
        }
        if (!isDataObject(item)) break;

        const next = this.getNextPage(item.chapter_id, computed <= (horizontal ? this.width : this.height), item.complete ? {
          version_id: item.complete.version_id,
          page_id: item.complete.page.id
        } : undefined)
        if (!next) break;
        if (typeof next === "string") {
          pos.push({offset: computed, message: next})
        } else {
          next.offset = computed;
          pos.push(next);
        }
        if (computed <= (horizontal ? this.width : this.height)) {
          render = [...pos];
        }
        if (typeof next === "string") break;
        computed += horizontal ? this.calculateWidth(next.complete) : this.calculateHeight(next.complete)
      }
      computed = start;
      end = -negative * (horizontal ? this.width : this.height);
      while (computed > end) {
        const item = pos[pos.length - 1];
        if (item == "Unreachable" || item == "Unreachable2" || item == "Loading") {
          break;
        }
        if (!isDataObject(item)) break;
        const next = this.getPrevPage(item.chapter_id, false, item.complete ? {
          version_id: item.complete.version_id,
          page_id: item.complete.page.id
        } : undefined)
        if (!next) break;
        if (typeof next === "string") {
          neg.push({offset: computed, message: next})
        } else {
          next.offset = computed;
          neg.push(next);
        }
        if (typeof next === "string") break;
        computed -= horizontal ? this.calculateWidth(next.complete) : this.calculateHeight(next.complete)
      }
      neg.shift();
      return [render, neg.reverse(), pos]
    },
    calculateHeight(complete?: { version_id: string, page: Page }) {
      if (!complete) return this.width * 1;
      return complete.page.height / complete.page.width * this.width
    },
    calculateWidth(complete?: { version_id: string, page: Page }) {
      if (!complete) return this.height * 1;
      return complete.page.width / complete.page.height * this.height
    },
    getLRScroll() {
      const a = this.getScroll(3, 4, true);
      this.start_download([...a[0], ...a[1], ...a[2]])
      this.render_images = a[0]
    },
    setProgressScroll(offset: [number, number], horizontal: boolean) {
      if (offset[0] > 1000 || offset[1] > 1000) return;
      if (this.navbar) {
        this.navbar = false;
      }
      if (!this.info || this.page[1] == 0) return;
      const [x, y] = offset;
      if ((horizontal ? x : y) == 0) return
      const ch = this.getChapter(this.info.open_chapter)
      if (!ch) return;
      const pages = this.getPages(ch.versions, false);
      if (pages == "no_version") {
        return "No version"
      } else if (!pages) {
        return "Loading"
      }
      const full_height = Object.values(pages.pages).map(v => horizontal ? v.width / v.height : v.height / v.width).reduce((acc, num) => acc + num, 0) * (horizontal ? this.height : this.width);
      const height = this.info.progress * full_height + (horizontal ? x : y);
      if (height < 0) {
        const pc = this.getPrevChapter(ch.chapter_id);
        if (!pc) return;
        const pages = this.getPages(pc.versions, false);
        if (pages == "no_version") {
          return "No version"
        } else if (!pages) {
          return "Loading"
        }
        this.info.progress = 1;
        this.setChapter(pc.chapter_id, pc.chapter)
        this.setProgressScroll([height, height], horizontal)
      } else if (height > full_height) {
        const nc = this.getNextChapter(ch.chapter_id);
        if (!nc) return;
        const pages = this.getPages(nc.versions, false);
        if (pages == "no_version") {
          return "No version"
        } else if (!pages) {
          return "Loading"
        }
        this.info.progress = 0;
        this.setChapter(nc.chapter_id, nc.chapter)
        this.setProgressScroll([height - full_height, height - full_height], horizontal)
      } else {
        this.info.progress = height / full_height;
        this.reload()
      }
    },
    setChapter(chapter_id: string, chapter_number: number) {
      if (!this.info) return;
      history.replaceState(
          {},
          "",
          `/reader/${this.info.manga_id}/${chapter_id}/${chapter_number ?? 'unknown'}`
      )
      //this.$router.replace({
      //   params: {
      //    manga_id: this.info.manga_id,
      //    chapter_id: [chapter_id, chapter_number ?? 'unknown']
      //  }
      //})
      this.info.open_chapter = chapter_id;
    },
    async start_download(pages: Ret[]) {
      const images: ImageItem[] = []
      for (const page of pages) {
        if (page != 'Loading' && page != 'Unreachable' && page != 'Unreachable2' && this.info && isDataObject(page) && page.complete) {
          images.push({
            manga_id: this.info.manga_id,
            chapter_id: page.chapter_id,
            version_id: page.complete.version_id ?? '',
            page: page.complete.page.page,
            file_ext: page.complete.page.ext
          })
        }
      }
      await getImages(this.cache, images)
    },
    getSinglePage() {
      const pages = this.getPagesCount(3, 3)

      this.start_download([...pages[0], ...pages[1]]);
      this.render_images = [pages[1][0]]
    },
    getDoublePage() {
      const pages = this.getPagesCount(3, 3 + 1)
      this.start_download([...pages[0], ...pages[1]]);
      this.render_images = [pages[1][0], pages[1][1]]
    },
    getPagesCount(left: number, right: number): [Ret[], Ret[]] {
      const root_ = this.getRoot(false, true)
      let root: DataObject | "Unreachable" | "Unreachable2" | "Loading";
      if (typeof root_ === 'string') {
        root = root_;
      } else {
        root = root_ as DataObject;
      }

      const pages: Tuple = [right, [root], true];
      const lpages: Tuple = [left, [pages[1][0]], false];
      for (const [len, target, next] of [pages, lpages]) {
        for (let i = 0; i < len; i++) {
          let last = target[target.length - 1];
          if (last == "Unreachable" || last == "Unreachable2" || last == "Loading") {
            break;
          }
          let temp;
          last = last as DataObject;
          if (next) {
            temp = this.getNextPage(last.chapter_id, i == 0, last.complete ? {
              version_id: last.complete.version_id,
              page_id: last.complete.page.id
            } : undefined);
          } else {
            temp = this.getPrevPage(last.chapter_id, false, last.complete ? {
              version_id: last.complete.version_id,
              page_id: last.complete.page.id
            } : undefined);
          }
          if (!temp) break;
          target.push(temp);
        }
      }
      lpages[1].shift()
      return [lpages[1].reverse(), pages[1]]
    },
    getNextPage(chapter_id: string, watch: boolean, complete?: {
      version_id: string,
      page_id: string
    }): DataObject | null | "Loading" {
      if (complete) {
        let ch = this.getChapter(chapter_id)!
        let pages: "no_version" | {
          link?: string | null | undefined,
          pages: Record<string, { ext: string, height: number, id: string, width: number, page: number }>
        } | null = this.getPages(ch!.versions, watch);
        if (!pages || pages == "no_version") {
          return "Loading"
        }
        let next = Object.values(pages.pages).find((page) => page.id === complete.page_id!)!.page + 1
        const nextPage = pages.pages[String(next)]
        if (!nextPage) {
          let ch_temp = this.getNextChapter(ch!.chapter_id)
          if (!ch_temp) {
            return null
          }
          pages = this.getPages(ch_temp.versions, watch);
          const version_id = this.getPreferedVersion(ch_temp.versions)!
          if (!pages || pages == "no_version") {
            return "Loading"
          }
          return {
            chapter_id: ch_temp.chapter_id,
            offset: 0,
            complete: {version_id: version_id, page: pages.pages[String(1)]}
          }
        }
        const version_id = this.getPreferedVersion(ch.versions)!
        return {chapter_id: ch.chapter_id, offset: 0, complete: {version_id: version_id, page: nextPage}}
      }
      let ch = this.getNextChapter(chapter_id);
      if (!ch) return null;
      let pages: {
        link?: string | null | undefined,
        pages: Record<string, { ext: string, height: number, id: string, width: number, page: number }>
      } | "no_version" | null = this.getPages(ch!.versions, watch);
      if (!pages || pages == "no_version") {
        return "Loading"
      }
      const nextPage = pages.pages[String(1)]
      if (!nextPage) {
        return null
      }
      const v_id = this.getPreferedVersion(ch.versions)!
      return {chapter_id: ch.chapter_id, offset: 0, complete: {version_id: v_id, page: nextPage}}
    },
    getPrevPage(chapter_id: string, watch: boolean, complete?: {
      version_id: string,
      page_id: string
    }): DataObject | null | "Loading" {
      if (complete) {
        let ch = this.getChapter(chapter_id)!
        let pages: "no_version" | {
          link?: string | null | undefined,
          pages: Record<string, { ext: string, height: number, id: string, width: number, page: number }>
        } | null = this.getPages(ch!.versions, watch);
        if (!pages || pages == "no_version") {
          return "Loading"
        }
        let prev = Object.values(pages.pages).find((page) => page.id === complete.page_id!)!.page - 1

        const prevPage = pages.pages[String(prev)]
        if (!prevPage) {
          let ch_temp = this.getPrevChapter(ch!.chapter_id)
          if (!ch_temp) {
            return null
          }
          pages = this.getPages(ch_temp.versions, watch);
          if (!pages || pages == "no_version") {
            return "Loading"
          }
          const v_id = this.getPreferedVersion(ch.versions)!
          const max = Math.max(...Object.values(pages.pages).map(v => v.page))
          return {
            chapter_id: ch_temp.chapter_id,
            offset: 0,
            complete: {
              version_id: v_id,
              page: pages.pages[String(max)]
            }
          }
        }
        const v_id = this.getPreferedVersion(ch.versions)!

        return {chapter_id: ch.chapter_id, offset: 0, complete: {version_id: v_id, page: prevPage}}
      }
      let ch = this.getPrevChapter(chapter_id);
      if (!ch) return null;
      let pages: {
        link?: string | null | undefined,
        pages: Record<string, { ext: string, height: number, id: string, width: number, page: number }>
      } | "no_version" | null = this.getPages(ch!.versions, watch);
      if (!pages || pages == "no_version") {
        return "Loading"
      }
      const max = Math.max(...Object.values(pages.pages).map(v => v.page))
      const prevPage = pages.pages[String(max)]
      if (!prevPage) {
        return null
      }
      const v_id = this.getPreferedVersion(ch.versions)!
      return {chapter_id: ch.chapter_id, offset: 0, complete: {version_id: v_id, page: prevPage}}
    },
    getRoot(horizontal: boolean, paginated: boolean): Ret {
      if (!this.info) return "Unreachable";
      const root = this.getChapter(this.info.open_chapter)
      if (!root) return "Unreachable";
      const pages = this.getPages(root.versions, true);
      if (pages == "no_version") {
        return {chapter_id: root.chapter_id, offset: 0}
      } else if (!pages) {
        return "Loading"
      }
      if (this.info.progress === 0.0) {
        const maxPage = Math.max(...Object.values(pages.pages).map(v => v.page))
        this.page = [1, maxPage]
        return {
          chapter_id: root.chapter_id,
          complete: {
            version_id: this.getPreferedVersion(root.versions)!,
            page: pages.pages[(String(1))],
          },
          offset: 0.0
        }
      }
      if (paginated) {
        const maxPage = Math.max(...Object.values(pages.pages).map(v => v.page))
        const current = Math.round(this.info.progress * (maxPage - 1)) + 1;
        this.page = [current, maxPage]
        return {
          chapter_id: root.chapter_id,
          complete: {
            version_id: this.getPreferedVersion(root.versions)!,
            page: pages.pages[(String(current))]
          },
          offset: 0.0
        }
      }
      const full_height_ratio_ = Object.values(pages.pages)
          .map(v => horizontal
              ? new Decimal(v.width).div(v.height)
              : new Decimal(v.height).div(v.width));
      const full_height_ratio = full_height_ratio_
          .reduce((acc, num) => acc.plus(num), new Decimal(0));

      let accumulated_ratio = new Decimal(0);

      for (const page of Object.values(pages.pages)
          .sort((a, b) => a.page - b.page)) {

        const page_ratio = horizontal
            ? new Decimal(page.width).div(page.height)
            : new Decimal(page.height).div(page.width);

        const next_accumulated_ratio = accumulated_ratio.plus(page_ratio);
        if (new Decimal(this.info.progress).lte(next_accumulated_ratio.div(full_height_ratio))) {

          //TODO: convert this.info.progress to decimal
          //(progress - (accumulated_ratio / full_height_ratio)) / ((next_accumulated_ratio / full_height_ratio) - (accumulated_ratio / full_height_ratio))
          //Better: (progress * full_height_ratio - accumulated_ratio) / (next_accumulated_ratio - accumulated_ratio)
          const relative_progress = new Decimal(this.info.progress).mul(full_height_ratio).sub(accumulated_ratio).div(next_accumulated_ratio.sub(accumulated_ratio));

          const maxPage = Math.max(...Object.values(pages.pages).map(v => v.page));
          this.page = [page.page, maxPage];
          return {
            chapter_id: root.chapter_id,
            complete: {
              version_id: this.getPreferedVersion(root.versions)!,
              page: page,
            },
            offset: relative_progress.toNumber(),
          };
        }

        accumulated_ratio = next_accumulated_ratio;
      }
      return "Unreachable2";
    },
    setProgress(pageNr: number, horizontal: boolean, paginated: boolean) {
      if (!this.info) return "Unreachable";
      const root = this.getChapter(this.info.open_chapter)
      if (!root) return "Unreachable";
      if (pageNr < 1) {
        const next = this.getPrevChapter(root.chapter_id)
        if (next) {
          this.setChapter(next.chapter_id, next.chapter);
          this.info.progress = 1.0;
          this.reload()
        }
        return
      }
      const pages = this.getPages(root.versions, false);
      if (pages == "no_version") {
        return "No version"
      } else if (!pages) {
        return "Loading"
      }
      if (pageNr > this.page[1]) {
        const next = this.getNextChapter(root.chapter_id)
        if (next) {
          this.setChapter(next.chapter_id, next.chapter);
          this.info.progress = 0.0;
          this.reload()
        }
        return;
      }
      if (paginated) {
        this.info.progress = (pageNr - 1) / (this.page[1] - 1);
      } else {
        const full_height_ratio = Object.values(pages.pages).map(v => horizontal ? v.width / v.height : v.height / v.width).reduce((acc, num) => acc + num, 0);

        const partial_ratio = Object.values(pages.pages).sort((a, b) => a.page - b.page).filter((k) => k.page <= pageNr).map(v => horizontal ? v.width / v.height : v.height / v.width).reduce((acc, num) => acc + num, 0);

        this.info.progress = partial_ratio / full_height_ratio;
      }

      this.reload()
    },
    getPages(versions: { [key: string]: string; }, watch: boolean) {
      const prefered_version_join = this.getPreferedVersionJoin(versions);
      if (!prefered_version_join) {
        return "no_version"
      } else {
        const pages = this.getPagesLocal(prefered_version_join)
        if (!pages) {
          this.getPagesServer(prefered_version_join, watch)
          return null;
        } else {
          return pages;
        }
      }
    },
    getPreferedVersionJoin(versions: { [p: string]: string }) {
      const hierachy = ["a", "b", "c"];
      for (const v of hierachy) {
        const version = versions[v]
        if (version) {
          return version;
        }
      }
      const f_elem = Object.entries(versions)[0];
      if (!f_elem) return null;
      return f_elem[1]
    },
    getPreferedVersion(versions: { [p: string]: string }) {
      const hierachy = ["chapter_versions:a", "chapter_versions:b", "chapter_versions:c"];
      for (const v of hierachy) {
        const version = versions[v]
        if (version) {
          return version.replace("chapter_versions:", "");
        }
      }
      const f_elem = Object.entries(versions)[0];
      if (!f_elem) return null;
      return f_elem[0].replace("chapter_versions:", "")
    },
    getPagesLocal(chapter_version_join_id: string) {
      if (chapter_version_join_id in this.img_info_map) {
        const item = this.img_info_map[chapter_version_join_id]
        if (item == "loading") return null;
        return item
      } else {
        return null;
      }
    },
    async getPagesServer(chapter_version_join_id: string, watch: boolean) {
      {
        if (this.img_info_map[chapter_version_join_id] == "loading") return null;
        this.img_info_map[chapter_version_join_id] = "loading";
        const {$manRead} = useNuxtApp();
        const data = await $manRead('/api/v1/chapter-versions/info', {
          method: "POST", body: {id: chapter_version_join_id},
          headers: {Authorization: `Bearer ${await getAccessToken()}`}
        });
        this.img_info_map[chapter_version_join_id] = data;
        if (watch) {
          //TODO: watch should be callback
          this.reload()
        }
        return data;
      }
    },
    getPrevChapter(chapter_id: string) {
      if (!this.info) return null;

      const currentIndex = this.info.chapters.findIndex((ch) => ch.chapter_id === chapter_id);

      if (currentIndex === 0) return null;

      return this.info.chapters[currentIndex - 1];
    },
    getChapter(chapter_id: string) {
      if (!this.info) return null;
      return this.info.chapters.find((ch) => ch.chapter_id === chapter_id);
    },
    getNextChapter(chapter_id: string) {
      if (!this.info) return null;

      const currentIndex = this.info.chapters.findIndex((ch) => ch.chapter_id === chapter_id);

      if (currentIndex === this.info.chapters.length - 1) return null;

      return this.info.chapters[currentIndex + 1];
    },
    rootToPage(page: Ret) {
      if (typeof page === "string") {
        return null;
      }
      const complete = (page as DataObject).complete;
      if (!complete) return null;
      return complete.page.page
    }
  },
  computed: {
    containerStyle() {
      const items = [`--image-blur: ${this.imageSettings.Blur}px; --image-brightness: ${this.imageSettings.Brightness}%;--image-contrast:${this.imageSettings.Contrast}%;--image-grayscale:${this.imageSettings.Grayscale}%;--image-invert:${this.imageSettings.Invert}%;--image-hue-rotate:${this.imageSettings['Hue-rotate']}deg;--image-saturate:${this.imageSettings.Saturate}%;--image-sepia:${this.imageSettings.Sepia}%;margin:auto;`];
      if (this.sizeMode === 'l') {
        items.push("height:100%; width:100%;");
        const w = `max-width: ${this.max_width}px;`;
        const h = `max-height: ${this.max_height}px;`;
        if (this.mode == 'v') {
          items.push(w);
        }else if (this.mode == 'h') {
          items.push(h);
        }else {
          items.push(h);
          items.push(w);
        }
      } else if (this.sizeMode === 'm') {
        const w = `width: calc(100% - ${this.reduce_width}px);`;
        const h = `height: calc(100% - ${this.reduce_width}px);`;
        if (this.mode == 'v') {
          items.push(w);
          items.push("height:100%;");
        }else if (this.mode == 'h') {
          items.push(h);
          items.push("width:100%;");
        }else {
          items.push(h);
          items.push(w);
        }
      } else {
        const w = `width:${this.percentage_width}%;`;
        const h = `height: ${this.percentage_height}%;`;
        if (this.mode == 'v') {
          items.push(w);
          items.push("height:100%;");
        }else if (this.mode == 'h') {
          items.push(h);
          items.push("width:100%;");
        }else {
          items.push(h);
          items.push(w);
        }
      }
      return items.join("");
    },
    processedMode: {
      get() {
        return this.mode;
      },
      set(value: 's' | 'd' | 'v' | 'h') {
        switch ([this.mode, value]) {
          case ['d', 's']:
          case ['d', 'd']:
          case ['s', 's']:
          case ['s', 'd']:
          case ['v', 'v']:
          case ['h', 'h']:
            break
          case ['s', 'v']:
          case ['d', 'v']:
            this.setProgress(this.page[0], false, false)
            break
          case ['v', 's']:
          case ['v', 'd']:
            this.setProgress(this.page[0], false, true)
            break
          case ['s', 'h']:
          case ['d', 'h']:
            const r3 = this.rootToPage(this.getRoot(false, true));
            if (!r3) return;
            this.setProgress(this.page[0], true, false)
            break
          case ['h', 's']:
          case ['h', 'd']:
            const r4 = this.rootToPage(this.getRoot(true, false));
            if (!r4) return;
            this.setProgress(this.page[0], false, true)
            break
        }
        this.mode = value;
        this.reload();
      },
    },
    active_chapter() {
      if (!this.value) return null;
      const open = this.value.open_chapter;
      return this.value.chapters.find(v => v.chapter_id == open);
    }
  },
  watch: {
    width: function () {
      this.reload();
    },
    height: function () {
      this.reload();
    },
  },
}
</script>
