<script lang="ts">
import { parse_query } from "search-parser";
import { Menu, MenuButton, MenuItem, MenuItems } from "@headlessui/vue";
import Pagination from "~/components/Pagination.vue";
import Warn from "~/components/hints/Warn.vue";
import { getTitle, navigateTo } from "#imports";
import type { NuxtError } from "#app";
import Sidebar from "~/components/SideBar.vue";
import CoverGet from "~/components/img/CoverGet.vue";

export default {
    components: {
        CoverGet,
        Sidebar,
        Warn,
        Pagination,
        Menu,
        MenuItem,
        MenuItems,
        MenuButton,
    },
    methods: {
        getTitle,
        navigateTo,
        async load_external() {
            if (this.params.mode == "Internal") {
                this.error = "selected Internal & external called";
                return;
            }
            const selected =
                this.search_stats?.stats.info[this.params.mode].search;
            if (!selected) {
                this.error = "unknown uri";
                return;
            }
            if (selected === "QueryOffset") {
            } else {
                this.error = JSON.stringify(selected) + " not implemented";
                return;
            }
            try {
                this.abortController.abort();
                this.abortController = new AbortController();
                const { $manRead } = useNuxtApp();
                this.search_stats!.external = await $manRead(
                    "/api/v1/external/search",
                    {
                        method: "POST",
                        headers: {
                            Authorization: `Bearer ${await getAccessToken()}`,
                        },
                        signal: this.abortController.signal,
                        body: {
                            uri: this.params.mode,
                            query: {
                                Simple: {
                                    query: this.params.query,
                                    page: this.params.page,
                                },
                            },
                        },
                    },
                );
                this.error = null;
                const div = this.$refs["search-container"];
                div.scrollTo(0, 0);
            } catch (e: any) {
                this.error = e;
            }
        },
        async load_internal() {
            try {
                this.abortController.abort();
                this.abortController = new AbortController();
                const { $manRead } = useNuxtApp();
                this.search_stats!.search = await $manRead(
                    "/api/v1/manga/search",
                    {
                        method: "POST",
                        headers: {
                            Authorization: `Bearer ${await getAccessToken()}`,
                        },
                        signal: this.abortController.signal,
                        body: {
                            desc: this.params.desc,
                            limit: 50,
                            order: this.params.order,
                            page: this.params.page,
                            query: JSON.parse(
                                parse_query(this.params.query, true),
                            ),
                        },
                    },
                );
                this.error = null;
                const div = this.$refs["search-container"];
                div.scrollTo(0, 0);
            } catch (e: any) {
                this.error = e;
            }
        },
    },

    mounted() {
        const router = useRouter();

        watch(
            this.params,
            (newParams) => {
                router.replace({
                    query: { ...newParams, desc: newParams.desc.toString() },
                });
                if (this.params.mode == "Internal") {
                    this.load_internal();
                } else {
                    this.load_external();
                }
            },
            { deep: true },
        );
    },
    computed: {
        internal() {
            return this.params.mode === "Internal";
        },
        searches() {
            if (!this.search_stats) return ["Internal"];
            const k = Object.entries(this.search_stats.stats.info)
                .filter(([_, v]) => !!v.search)
                .map(([k, _]) => k)
                .filter((item) => item !== "Internal")
                .sort();
            return ["Internal", ...k];
        },
        lastPage() {
            if (!this.search_stats) return 1;
            return Math.max(
                this.search_stats!.search
                    ? Math.ceil(this.search_stats!.search.max / 50)
                    : 1,
                1,
            );
        },
    },
    async setup() {
        //TODO: infinite scroll, actions left & right
        //TODO: advanced search: https://dribbble.com/shots/22523843-Logic-filters https://dribbble.com/shots/10527249-Condition-Based-Advanced-Filters
        useHead({
            title: "Search - ManRead",
        });
        const route = useRoute();
        const params = ref({
            desc: route.query.desc === "true",
            order: String(route.query.order || "alphabetical"),
            page: Number(route.query.page) || 1,
            query: String(route.query.query || ""),
            mode: String(route.query.mode || "Internal"),
        });

        const abortController = new AbortController();
        const { data: value, error } = await useAsyncData(
            "home-data",
            async () => {
                const { $manRead } = useNuxtApp();
                const access = await getAccessToken();
                const search = await $manRead("/api/v1/manga/search", {
                    method: "POST",
                    body: {
                        desc: params.value.desc,
                        limit: 50,
                        order: params.value.order,
                        page: params.value.page,
                        query: JSON.parse(
                            parse_query(params.value.query, true),
                        ),
                    },
                    headers: { Authorization: `Bearer ${access}` },
                });
                const stats = await $manRead("/api/v1/external/statistics", {
                    method: "POST",
                    headers: { Authorization: `Bearer ${access}` },
                });
                return { search, stats, external: [] };
            },
        );
        const err: Ref<null | NuxtError<unknown> | string> = error;
        return {
            abortController,
            isFocused: ref(false),
            search_stats: value,
            error: err,
            params,
        };
    },
};
</script>

<template>
    <div class="flex h-full w-full">
        <div class="mx-2 mt-2 items-center space-x-2 h-full flex grow flex-col">
            <div
                class="flex w-full rounded-md bg-white outline-1 -outline-offset-1 outline-gray-300"
                :class="{
                    'outline-2 -outline-offset-2 outline-indigo-600': isFocused,
                }"
            >
                <Menu
                    as="div"
                    class="select-none relative cursor-pointer flex px-3 sm:text-sm/6 border-r border-gray-300 py-1.5 z-2"
                >
                    <MenuButton
                        as="div"
                        class="flex justify-center items-center"
                    >
                        <p class="text-gray-900">{{ params.mode }}</p>
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            class="ml-1"
                            style="height: calc(var(--text-sm) * 1.25)"
                            viewBox="0 0 24 24"
                            fill="none"
                        >
                            <path
                                d="M7 10L12 15L17 10"
                                stroke="#000000"
                                stroke-width="1.5"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            />
                        </svg>
                    </MenuButton>
                    <transition
                        enter-active-class="transition duration-100 ease-out"
                        enter-from-class="transform scale-95 opacity-0"
                        enter-to-class="transform scale-100 opacity-100"
                        leave-active-class="transition duration-75 ease-in"
                        leave-from-class="transform scale-100 opacity-100"
                        leave-to-class="transform scale-95 opacity-0"
                    >
                        <MenuItems
                            class="absolute left-0 mt-2 w-56 origin-top-right divide-y divide-gray-100 rounded-md bg-white shadow-lg ring-1 ring-black/5 focus:outline-none max-h-[70vh] overflow-scroll"
                        >
                            <div
                                class="px-1 py-1"
                                v-for="key in searches"
                                :key="key"
                            >
                                <MenuItem v-slot="{ active }">
                                    <button
                                        @click="() => (params.mode = key)"
                                        :class="[
                                            active
                                                ? 'bg-violet-500 text-white'
                                                : 'text-gray-900',
                                            'group flex w-full items-center rounded-md px-2 py-2 text-sm',
                                        ]"
                                    >
                                        {{ key }}
                                    </button>
                                </MenuItem>
                            </div>
                        </MenuItems>
                    </transition>
                </Menu>
                <input
                    v-model="params.query"
                    placeholder="Search..."
                    @focus="isFocused = true"
                    @blur="isFocused = false"
                    class="w-full block px-3 py-1.5 text-base text-gray-900 placeholder:text-gray-400 sm:text-sm/6 focus:outline-0 outline-0"
                />
                <Menu
                    as="div"
                    class="relative cursor-pointer flex px-3 sm:text-sm/6 border-l border-gray-300 py-1.5 z-2"
                >
                    <MenuButton
                        as="div"
                        class="flex justify-center items-center"
                    >
                        <p class="text-gray-900">Internal</p>
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            class="ml-1"
                            style="height: calc(var(--text-sm) * 1.25)"
                            viewBox="0 0 24 24"
                            fill="none"
                        >
                            <path
                                d="M7 10L12 15L17 10"
                                stroke="#000000"
                                stroke-width="1.5"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            />
                        </svg>
                    </MenuButton>
                    <transition
                        enter-active-class="transition duration-100 ease-out"
                        enter-from-class="transform scale-95 opacity-0"
                        enter-to-class="transform scale-100 opacity-100"
                        leave-active-class="transition duration-75 ease-in"
                        leave-from-class="transform scale-100 opacity-100"
                        leave-to-class="transform scale-95 opacity-0"
                    >
                        <MenuItems
                            class="absolute right-0 mt-2 w-56 origin-top-right divide-y divide-gray-100 rounded-md bg-white shadow-lg ring-1 ring-black/5 focus:outline-none"
                        >
                            <div class="px-1 py-1">
                                <MenuItem v-slot="{ active }">
                                    <button
                                        :class="[
                                            active
                                                ? 'bg-violet-500 text-white'
                                                : 'text-gray-900',
                                            'group flex w-full items-center rounded-md px-2 py-2 text-sm',
                                        ]"
                                    >
                                        Edit
                                    </button>
                                </MenuItem>
                                <MenuItem v-slot="{ active }">
                                    <button
                                        :class="[
                                            active
                                                ? 'bg-violet-500 text-white'
                                                : 'text-gray-900',
                                            'group flex w-full items-center rounded-md px-2 py-2 text-sm',
                                        ]"
                                    >
                                        Duplicate
                                    </button>
                                </MenuItem>
                            </div>
                            <div class="px-1 py-1">
                                <MenuItem v-slot="{ active }">
                                    <button
                                        :class="[
                                            active
                                                ? 'bg-violet-500 text-white'
                                                : 'text-gray-900',
                                            'group flex w-full items-center rounded-md px-2 py-2 text-sm',
                                        ]"
                                    >
                                        Archive
                                    </button>
                                </MenuItem>
                                <MenuItem v-slot="{ active }">
                                    <button
                                        :class="[
                                            active
                                                ? 'bg-violet-500 text-white'
                                                : 'text-gray-900',
                                            'group flex w-full items-center rounded-md px-2 py-2 text-sm',
                                        ]"
                                    >
                                        Move
                                    </button>
                                </MenuItem>
                            </div>

                            <div class="px-1 py-1">
                                <MenuItem v-slot="{ active }">
                                    <button
                                        :class="[
                                            active
                                                ? 'bg-violet-500 text-white'
                                                : 'text-gray-900',
                                            'group flex w-full items-center rounded-md px-2 py-2 text-sm',
                                        ]"
                                    >
                                        Delete
                                    </button>
                                </MenuItem>
                            </div>
                        </MenuItems>
                    </transition>
                </Menu>
            </div>
            <div></div>
            <Warn v-if="error" color="red" :message="String(error)"></Warn>
            <div
                class="h-full w-full my-2 overflow-auto"
                ref="search-container"
                style="--search-grid-size: 240px"
            >
                <template v-if="search_stats">
                    <ul v-if="internal" class="grid-container">
                        <li
                            v-for="item in search_stats.search.items"
                            :key="item.manga_id"
                        >
                            <CoverGet
                                :manga_id="item.manga_id"
                                :ext="item.ext"
                                :status="item.status"
                                class-list="w-full h-[calc(100%-3rem)] object-cover"
                            />
                            <div
                                class="flex items-center justify-center h-[3rem] mx-1 cursor-pointer"
                                @click="
                                    () => navigateTo(`/info/${item.manga_id}`)
                                "
                            >
                                <p
                                    class="text-center text-gray-900 line-clamp-2 text-sm font-semibold text-ellipsis"
                                >
                                    {{ getTitle(item.titles) }}
                                </p>
                            </div>
                        </li>
                    </ul>
                    <ul v-else class="grid-container">
                        <li
                            v-for="item in search_stats.external"
                            :key="item.url"
                        >
                            <img
                                :src="item.cover"
                                class="w-full h-[calc(100%-3rem)] object-cover"
                            />
                            <div
                                class="flex items-center justify-center h-[3rem] mx-1 cursor-pointer"
                                @click="() => navigateTo(item.url)"
                            >
                                <p
                                    class="text-center text-gray-900 line-clamp-2 text-sm font-semibold text-ellipsis"
                                >
                                    {{ item.title }}
                                </p>
                            </div>
                        </li>
                    </ul>
                </template>

                <Pagination
                    :current_page="params.page"
                    :last_page="lastPage"
                    @change-page="(page) => (params.page = page)"
                />
            </div>
        </div>
        <Sidebar active-tab="search" />
    </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";
.grid-container {
    display: grid;
    grid-template-columns: repeat(
        auto-fill,
        minmax(var(--search-grid-size), 1fr)
    );
    gap: 12px;
    list-style-type: none;
    padding: 0;
}

.grid-container li {
    aspect-ratio: 3/5;
    border-radius: 0.25rem;
    background: white;
    text-align: center;
}
</style>
