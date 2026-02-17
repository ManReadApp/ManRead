<script setup lang="ts">
import DoubleFloatingContainer from "~/components/container/DoubleFloatingContainer.vue";
import FileUpload from "~/components/input/FileUpload.vue";
import Selector from "~/components/input/Selector.vue";
import MapDisplay from "~/components/label/MapDisplay.vue";
import RadioButtons from "~/components/input/RadioButtons.vue";
import { getAccessToken } from "~/utils/auth";
import TagRowDisplay from "~/components/label/TagRowDisplay.vue";
import PrimaryButton from "~/components/button/PrimaryButton.vue";

const { $manRead } = useNuxtApp();
const { data } = await useAsyncData("static-data", async () => {
    let accessToken = await getAccessToken();
    const chapter_versions = (
        await $manRead("/api/v1/chapter-versions/list", {
            method: "POST",
            body: { limit: 1000, page: 1 },
            headers: { Authorization: `Bearer ${accessToken}` },
        })
    ).map((v) => v.name);
    const kinds = (
        await $manRead("/api/v1/kind/list", {
            method: "POST",
            headers: { Authorization: `Bearer ${accessToken}` },
        })
    ).items;
    return { chapter_versions, kinds };
});

const langs = [
    "en", // English
    "jp", // Japanese
    "jp_latin",
    "cn", //Chinese
    "cn_latin",
    "kr", // Korean
    "kr_latin",
    "ru", // Russian
    "ru_latin",
    "ua", // Ukraine
    "ua_latin",
    "th", // Thailand
    "th_latin",
    "es", // Spanish
    "fr", // French
    "de", // German
    "vi", // Vietnamese
    "pl", // Polish
    "pt", // Portuguese
    "ar", // Arabic
    "ar_latin",
    "unknown",
];

const props = defineProps<{
    mangaId?: string;
    initialStatus?: "Dropped" | "Ongoing" | "Completed" | "Hiatus" | "Upcoming";
    initialKind?: string;
    initialDescription?: string;
    initialTitles?: Record<string, { items: string[] }>;
    initialPpls?: Record<string, { items: string[] }>;
    initialTags?: Record<ValidKey, { items: string[] }>;
    initialScrapers?: Record<string, { items: string[] }>;
    initialSources?: string[];
}>();

const hasScrapers =
    props.initialScrapers &&
    Object.values(props.initialScrapers).flatMap(({ items }) => items).length > 0;
type ValidKey = "♂" | "♀" | "⚤ | ⚥" | "♂ → ♀" | "♀ → ♂" | "⊗" | "?⃝";
const values = reactive<{
    file: string | null;
    lang: string;
    ppl: string;
    kind: string;
    ppls: Record<string, { items: string[] }>;
    tag: ValidKey;
    status: "Dropped" | "Ongoing" | "Completed" | "Hiatus" | "Upcoming" | "";
    chapter_version_select: string;
    titles_text: string;
    description: string;
    titles: Record<string, { items: string[] }>;
    tags_text: string;
    tags: Record<ValidKey, { items: string[] }>;
    scrapers: Record<string, { items: string[] }>;

    ppl_text: string;
    scraper: boolean;
    sources: string[];
    scrapers_text: string;
    sources_text: string;
}>({
    file: null,
    status: props.initialStatus ?? "",
    kind: props.initialKind ?? "",
    titles: props.initialTitles ?? {},
    scrapers: props.initialScrapers ?? {},
    sources: props.initialSources ?? [],
    tags: props.initialTags ?? {},
    ppls: props.initialPpls ?? {},
    description: props.initialDescription ?? "",

    chapter_version_select: "",
    tag: "♀",
    lang: "en",
    ppl: "Author",
    scraper: hasScrapers ?? false,

    tags_text: "",
    titles_text: "",
    ppl_text: "",
    scrapers_text: "",
    sources_text: "",
});

const validator = computed(() => {
    const file = props.mangaId || values.file;
    console.log(
        file,
        values.status,
        values.kind,
        !!Object.values(values.titles).flatMap(({ items }) => items).length,
        !values.scraper ||
            Object.values(values.scrapers).flatMap(({ items }) => items).length >
                0,
        // check empty text fields
        !values.tags_text,
        !values.titles_text,
        !values.ppl_text,
        !values.scrapers_text,
        !values.sources_text,
    );
    // check required fields
    return !!(
        file &&
        values.status &&
        values.kind &&
        !!Object.values(values.titles).flatMap(({ items }) => items).length &&
        (!values.scraper ||
            Object.values(values.scrapers).flatMap(({ items }) => items)
                .length > 0) &&
        // check empty text fields
        !values.tags_text &&
        !values.titles_text &&
        !values.ppl_text &&
        !values.scrapers_text &&
        !values.sources_text
    );
});

const chapter_versions = computed(() => {
    if (!data.value) return [];
    if (!data.value.chapter_versions) return [];
    return data.value.chapter_versions;
});
const kinds = computed(() => {
    if (!data.value) return [];
    if (!data.value.kinds) return [];
    return data.value.kinds;
});

const display_tags = computed(() => {
    return Array.from(Object.entries(values.tags)).flatMap(([key, values]) => {
        return values.items.map((value) => `${key} ${value}`);
    });
});

const remove_tag = (index: number, set = false) => {
    const str = display_tags.value[index];
    const [key_, _] = str.split(/\u200B(.*)/s);

    const validKeys = ["♂", "♀", "⚤", "⚥", "♂ → ♀", "♀ → ♂", "⊗", "?⃝"];

    if (!validKeys.includes(key_)) return;
    const key = key_ as ValidKey;
    const arr = values.tags[key]?.items ?? [];

    let count = 0;
    for (let i = 0; i < index; i++) {
        const [k] = display_tags.value[i].split(/\u200B(.*)/s);
        if (k === key) {
            count++;
        }
    }

    if (set) {
        values.tags_text = arr[count];
        values.tag = key;
    }

    if (count < arr.length) {
        arr.splice(count, 1);
    }
};

const userSearchCache = new Map<string, Ref<string[][]>>();
const tagSearchCache = new Map<string, Ref<string[]>>();

const search_tags = (name: string) => {
    if (tagSearchCache.has(name)) {
        return tagSearchCache.get(name);
    } else {
        const store = ref<string[]>([]);
        tagSearchCache.set(name, store);
        search_tags_online(store, name);
        return store;
    }
};

const tag_completion = computed(() => {
    if (values.tags_text.length == 0) return [];
    const prefix = `${values.tag}\u200B`;
    const temp = search_tags(values.tags_text);
    if (!temp) return [];
    if (!temp.value) return [];
    const startsWithPrefix = [];
    const others = [];

    for (const item of temp.value) {
        if (item.startsWith(prefix)) {
            startsWithPrefix.push(item);
        } else {
            others.push(item);
        }
    }

    return [...startsWithPrefix, ...others];
});

const create = async () => {
    if (!values.status || props.mangaId || !values.file) return;

    return await $manRead("/api/v1/manga/detail/create", {
        method: "PUT",
        body: {
            image_temp_name: values.file,
            artists: values.ppls["Artist"]?.items ?? [],
            authors: values.ppls["Author"]?.items ?? [],
            description: values.description || undefined,
            kind: values.kind,
            names: values.titles,
            publishers: values.ppls["Publisher"]?.items ?? [],
            scrapers: Object.entries(values.scrapers).flatMap(
                ([channel, { items }]) =>
                    items.map((url) => ({ channel, url })),
            ),
            sources: values.sources,
            status: values.status,
            tags: Object.entries(values.tags).flatMap(([kind, { items }]) => {
                let sex:
                    | "Male"
                    | "Female"
                    | "Both"
                    | "None"
                    | "FemaleMale"
                    | "MaleFemale"
                    | "Unknown" = "Male";
                switch (kind) {
                    case "♂":
                        sex = "Male";
                        break;
                    case "♀":
                        sex = "Female";
                        break;
                    case "⚤ | ⚥":
                        sex = "Both";
                        break;
                    case "♂ → ♀":
                        sex = "MaleFemale";
                        break;
                    case "♀ → ♂":
                        sex = "FemaleMale";
                        break;
                    case "⊗":
                        sex = "None";
                        break;
                    case "?⃝":
                        sex = "Unknown";
                        break;
                }
                return items.map((tag) => ({ tag, sex, description: null }));
            }),
        },
        headers: { Authorization: `Bearer ${await getAccessToken()}` },
    });
};

const submit = async () => {
    let id;
    if (props.mangaId) {
        id = await update();
    } else {
        id = await create();
    }
    navigateTo(`/info/${id}/`);
};

const update = async () => {
    if (!values.status || !props.mangaId) return;
    await $manRead("/api/v1/manga/detail/edit", {
        method: "PUT",
        body: {
            artists: values.ppls["Artist"]?.items ?? [],
            authors: values.ppls["Author"]?.items ?? [],
            description: values.description || undefined,
            kind: values.kind,
            manga_id: props.mangaId,
            names: values.titles,
            publishers: values.ppls["Publisher"]?.items ?? [],
            scrapers: Object.entries(values.scrapers).flatMap(
                ([channel, { items }]) =>
                    items.map((url) => ({ channel, url })),
            ),
            sources: values.sources,
            status: values.status,
            tags: Object.entries(values.tags).flatMap(([kind, { items }]) => {
                let sex:
                    | "Male"
                    | "Female"
                    | "Both"
                    | "None"
                    | "FemaleMale"
                    | "MaleFemale"
                    | "Unknown" = "Male";
                switch (kind) {
                    case "♂":
                        sex = "Male";
                        break;
                    case "♀":
                        sex = "Female";
                        break;
                    case "⚤ | ⚥":
                        sex = "Both";
                        break;
                    case "♂ → ♀":
                        sex = "MaleFemale";
                        break;
                    case "♀ → ♂":
                        sex = "FemaleMale";
                        break;
                    case "⊗":
                        sex = "None";
                        break;
                    case "?⃝":
                        sex = "Unknown";
                        break;
                }
                return items.map((tag) => ({ tag, sex, description: null }));
            }),
        },
        headers: { Authorization: `Bearer ${await getAccessToken()}` },
    });
    return props.mangaId;
};

const search_tags_online = async (store: Ref<string[]>, name: string) => {
    const data = await $manRead("/api/v1/tags/search", {
        method: "POST",
        body: { limit: 9999, page: 1, query: name },
        headers: { Authorization: `Bearer ${await getAccessToken()}` },
    });
    let query = name.toLowerCase();
    store.value = data
        .sort((a, b) => {
            const aTag = a.tag.toLowerCase();
            const bTag = b.tag.toLowerCase();

            const aStarts = aTag.startsWith(query);
            const bStarts = bTag.startsWith(query);

            if (aStarts && !bStarts) return -1;
            if (!aStarts && bStarts) return 1;

            return aTag.localeCompare(bTag);
        })
        .map((v) => {
            let p = "";
            switch (v.sex) {
                case "Female":
                    p = "♀";
                    break;
                case "Male":
                    p = "♂";
                    break;
                case "Both":
                    p = "⚤ | ⚥";
                    break;
                case "None":
                    p = "⊗";
                    break;
                case "FemaleMale":
                    p = "♀ → ♂";
                    break;
                case "MaleFemale":
                    p = "♂ → ♀";
                    break;
                case "Unknown":
                    p = "?⃝";
                    break;
            }
            return `${p}\u200B ${v.tag}`;
        });
};

const search_users = (name: string) => {
    if (userSearchCache.has(name)) {
        return userSearchCache.get(name);
    } else {
        const store = ref<string[][]>([]);
        userSearchCache.set(name, store);
        search_users_online(store, name);
        return store;
    }
};

const user_completion = computed(() => {
    if (values.ppl_text.length == 0) return [];
    const temp = search_users(values.ppl_text);
    if (!temp) return [];
    return temp.value.map((v) => v[0]);
});

const search_users_online = async (store: Ref<string[][]>, name: string) => {
    const data = await $manRead("/api/v1/user/search", {
        method: "POST",
        body: { limit: 100, page: 1, query: name },
        headers: { Authorization: `Bearer ${await getAccessToken()}` },
    });
    store.value = data.map((v) => v.names);
};
</script>
<template>
    <DoubleFloatingContainer>
        <template #main>
            <h2 class="text-2xl font-bold mb-2">Add Manga</h2>
            <FileUpload
                v-if="!mangaId"
                @uploaded="(d: string | null) => (values.file = d)"
                class="mb-2"
            />
            <RadioButtons
                :categories="[
                    'Ongoing',
                    'Completed',
                    'Hiatus',
                    'Upcoming',
                    'Dropped',
                ]"
                :colors="[
                    'bg-[#04D000]/40 border-[#04D000]',
                    'bg-[#00C9F5]/40 border-[#00C9F5]',
                    'bg-[#DA7500]/40 border-[#DA7500]',
                    'bg-[#7D40FF]/40 border-[#7D40FF]',
                    'bg-[#de3b3b]/40 border-[#de3b3b]',
                ]"
                v-model="values.status"
                class="grow mb-2"
            />

            <InputFieldBase
                label="Titles"
                id="titles"
                :disabled="false"
                type="text"
                auto-complete="off"
                v-model="values.titles_text"
                @pressed:enter="
                    () => {
                        (values.titles[values.lang] ??= { items: [] }).items.push(
                            values.titles_text,
                        );
                        values.titles_text = '';
                    }
                "
            >
                <template #no-flex>
                    <MapDisplay
                        :data="values.titles"
                        class="w-full mb-1"
                        @tag:dblclick="
                            (v) => {
                                const key = v.key;
                                const index = v.value;
                                const value = values.titles[key].items[index];
                                values.titles[key].items.splice(index, 1);
                                values.lang = key;
                                values.titles_text = value;
                            }
                        "
                        @tag:remove="
                            (v) => {
                                const key = v.key;
                                const index = v.value;
                                values.titles[key].items.splice(index, 1);
                            }
                        "
                    />
                </template>
                <template #flex>
                    <Selector
                        :suggestions="langs"
                        v-model="values.lang"
                        class="min-w-27 mr-2"
                    />
                </template>
            </InputFieldBase>

            <InputFieldBase
                label="Description"
                id="description"
                :disabled="false"
                type="textarea"
                auto-complete="off"
                v-model="values.description"
            />
            <InputFieldBase
                label="Tags"
                id="tags"
                :disabled="false"
                type="text"
                auto-complete="off"
                v-model="values.tags_text"
                :completions="tag_completion"
                @pressed:enter="
                    () => {
                        (values.tags[values.tag] ??= { items: [] }).items.push(
                            values.tags_text,
                        );
                        values.tags_text = '';
                    }
                "
            >
                <template #no-flex>
                    <TagRowDisplay
                        :values="display_tags"
                        @tag:remove="remove_tag"
                        @tag:dblclick="(i) => remove_tag(i, true)"
                    />
                </template>
                <template #flex>
                    <Selector
                        :suggestions="[
                            '♂',
                            '♀',
                            '⚤ | ⚥',
                            '♂ → ♀',
                            '♀ → ♂',
                            '⊗',
                            '?⃝',
                        ]"
                        v-model="values.tag"
                        class="min-w-27 mr-2"
                    />
                </template>
            </InputFieldBase>

            <InputFieldBase
                label="Creators"
                id="authors"
                :disabled="false"
                type="text"
                auto-complete="off"
                v-model="values.ppl_text"
                :completions="user_completion"
                @pressed:enter="
                    () => {
                        (values.ppls[values.ppl] ??= { items: [] }).items.push(
                            values.ppl_text,
                        );
                        values.ppl_text = '';
                    }
                "
            >
                <template #no-flex>
                    <MapDisplay
                        :data="values.ppls"
                        v-if="
                            !!Object.values(values.ppls).flatMap(({ items }) => items)
                                .length
                        "
                        class="w-full mb-1"
                        @tag:dblclick="
                            (v) => {
                                const key = v.key;
                                const index = v.value;
                                const value = values.ppls[key].items[index];
                                values.ppls[key].items.splice(index, 1);
                                values.ppl = key;
                                values.ppl_text = value;
                            }
                        "
                        @tag:remove="
                            (v) => {
                                const key = v.key;
                                const index = v.value;
                                values.ppls[key].items.splice(index, 1);
                            }
                        "
                    />
                </template>
                <template #flex>
                    <Selector
                        :suggestions="['Author', 'Artist', 'Publisher']"
                        v-model="values.ppl"
                        class="min-w-27 mr-2"
                    />
                </template>
            </InputFieldBase>

            <InputFieldBase
                :disabled="false"
                label="Sources"
                id="sources"
                type="text"
                auto-complete="off"
                v-model="values.sources_text"
                @pressed:enter="
                    () => {
                        values.sources.push(values.sources_text);
                        values.sources_text = '';
                    }
                "
            >
                <template #no-flex>
                    <TagRowDisplay
                        :values="values.sources"
                        @tag:remove="(i) => values.sources.splice(i, 1)"
                        @tag:dblclick="
                            (i) => {
                                values.sources_text = values.sources[i];
                                values.sources.splice(i, 1);
                            }
                        "
                    />
                </template>
            </InputFieldBase>
            <div class="flex">
                <div class="grow">
                    <InputFieldBase
                        label="Kind"
                        id="kind"
                        :disabled="false"
                        type="text"
                        v-model="values.kind"
                        auto-complete="off"
                        :completions="kinds"
                        class="grow"
                        :open-upwards="true"
                        :open-button="true"
                    />
                </div>
                <div class="flex flex-col items-end">
                    <p class="block text-sm/6 font-medium text-gray-900">
                        Scraper
                    </p>
                    <label
                        class="relative inline-flex items-center cursor-pointer mt-2"
                    >
                        <input
                            type="checkbox"
                            class="checkbox appearance-none w-8 h-8 rounded-md bg-white outline-1 outline-gray-300 checked:bg-indigo-600 focus:outline-2 focus:outline-indigo-600"
                            v-model="values.scraper"
                        />
                    </label>
                </div>
            </div>
            <InputFieldBase
                label="Scrapers"
                id="scrapers"
                :disabled="false"
                type="text"
                auto-complete="off"
                v-model="values.scrapers_text"
                v-if="values.scraper"
                @pressed:enter="
                    () => {
                        if (values.chapter_version_select.length > 0) {
                            (values.scrapers[values.chapter_version_select] ??=
                                { items: [] }).items.push(values.scrapers_text);
                            values.scrapers_text = '';
                        }
                    }
                "
            >
                <template #no-flex>
                    <MapDisplay
                        :data="values.scrapers"
                        class="w-full mb-1"
                        @tag:dblclick="
                            (v) => {
                                const key = v.key;
                                const index = v.value;
                                const value = values.scrapers[key].items[index];
                                values.scrapers[key].items.splice(index, 1);
                                values.chapter_version_select = key;
                                values.scrapers_text = value;
                            }
                        "
                        @tag:remove="
                            (v) => {
                                const key = v.key;
                                const index = v.value;
                                values.scrapers[key].items.splice(index, 1);
                            }
                        "
                    />
                </template>
                <template #flex>
                    <Selector
                        :suggestions="chapter_versions"
                        v-model="values.chapter_version_select"
                        class="min-w-36 mr-2"
                        :open-upwards="true"
                    />
                </template>
            </InputFieldBase>
            <PrimaryButton
                class="mt-4"
                :enabled="validator"
                name="Add Manga"
                @click="submit"
            />
        </template>

        <template #info>
            <h2 class="text-2xl font-bold">Info</h2>
        </template>
    </DoubleFloatingContainer>
</template>
<style scoped>
.checkbox:checked::after {
    content: "✓";
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-indigo-600);
    font-size: 1.5rem;
    font-weight: bold;
}

.checkbox:checked {
    background-color: transparent;
}
</style>
