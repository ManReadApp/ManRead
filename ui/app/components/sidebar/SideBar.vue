<template>
    <div
        class="relative h-screen border-l border-slate-200 bg-white shadow-lg transition-all duration-300 ease-in-out dark:border-slate-800 dark:bg-slate-950"
        :class="isExpanded ? 'w-64' : 'w-20'"
    >
        <div class="flex h-full flex-col px-3 py-3">
            <div
                class="flex mb-6"
                :class="isExpanded ? 'justify-start' : 'justify-center'"
            >
                <button
                    class="rounded-lg p-2 text-slate-600 transition hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-900"
                    @click="isExpanded = !isExpanded"
                >
                    <HamburgerIcon class="w-6 h-6" />
                </button>
            </div>

            <nav class="flex-1 space-y-2">
                <div v-for="item in menuItems" :key="item.name">
                    <button
                        class="group flex w-full items-center rounded-lg px-3 py-2 transition-colors duration-200"
                        :class="[
                            activeTab === item.name
                                ? 'bg-indigo-100 text-indigo-700 dark:bg-indigo-500/20 dark:text-indigo-200'
                                : 'text-slate-600 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-900',
                            isExpanded ? 'justify-start' : 'justify-center',
                        ]"
                        @click="$router.push(item.path)"
                    >
                        <component
                            :is="item.icon"
                            class="w-6 h-6 flex-shrink-0"
                        />
                        <span
                            class="overflow-hidden transition-all duration-300 whitespace-nowrap"
                            :class="
                                isExpanded
                                    ? 'ml-3 max-w-full opacity-100'
                                    : 'ml-0 max-w-0 opacity-0'
                            "
                        >
                            {{ item.label }}
                        </span>
                    </button>
                </div>
            </nav>

            <div class="pt-3">
                <button
                    class="group flex w-full items-center rounded-lg px-3 py-2 text-slate-600 transition-colors duration-200 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-900"
                    :class="isExpanded ? 'justify-start' : 'justify-center'"
                    @click="$router.push('/settings')"
                >
                    <SettingsIcon class="w-6 h-6 flex-shrink-0" />
                    <span
                        class="overflow-hidden transition-all duration-300 whitespace-nowrap"
                        :class="
                            isExpanded
                                ? 'ml-3 max-w-full opacity-100'
                                : 'ml-0 max-w-0 opacity-0'
                        "
                    >
                        Settings
                    </span>
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import SettingsIcon from "~/components/icons/SettingsIcon.vue";
import SearchIcon from "~/components/icons/SearchIcon.vue";
import HomeIcon from "~/components/icons/HomeIcon.vue";
import ListIcon from "~/components/icons/ListIcon.vue";
import HamburgerIcon from "~/components/icons/HamburgerIcon.vue";
import FolderDown from "~/components/icons/FolderDown.vue";
import ScrollText from "~/components/icons/ScrollText.vue";
import BadgeCheck from "~/components/icons/BadgeCheck.vue";

defineProps<{ activeTab: string }>();

const isExpanded = ref(false);
const menuItems = [
    { name: "home", label: "Home", icon: HomeIcon, path: "/" },
    { name: "search", label: "Search", icon: SearchIcon, path: "/search" },
    { name: "mylist", label: "My List", icon: ListIcon, path: "/mylist" },
    {
        name: "scraper",
        label: "Scraper",
        icon: FolderDown,
        path: "/download-manager",
    },
    {
        name: "tokens",
        label: "Tokens",
        icon: BadgeCheck,
        path: "/token-manager",
    },
    {
        name: "logs",
        label: "Logs",
        icon: ScrollText,
        path: "/logs",
    },
];
</script>
