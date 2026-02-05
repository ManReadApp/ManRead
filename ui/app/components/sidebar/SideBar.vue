<template>
    <div
        class="h-screen bg-white shadow-lg relative transition-all duration-300 ease-in-out"
        :class="isExpanded ? 'w-64' : 'w-20'"
    >
        <div class="flex flex-col h-full p-4">
            <div
                class="flex mb-8"
                :class="isExpanded ? 'justify-start' : 'justify-center'"
            >
                <button
                    class="p-2 rounded-lg hover:bg-gray-100 text-gray-600"
                    @click="isExpanded = !isExpanded"
                >
                    <HamburgerIcon class="w-6 h-6" />
                </button>
            </div>

            <nav class="flex-1 space-y-4">
                <div v-for="item in menuItems" :key="item.name">
                    <button
                        class="flex items-center w-full p-3 rounded-lg transition-colors duration-200 group"
                        :class="[
                            activeTab === item.name
                                ? 'bg-blue-100 text-blue-600'
                                : 'hover:bg-gray-100 text-gray-600',
                        ]"
                        @click="$router.push(item.path)"
                    >
                        <component
                            :is="item.icon"
                            class="w-6 h-6 flex-shrink-0"
                        />
                        <span
                            class="ml-3 overflow-hidden transition-all duration-300 whitespace-nowrap"
                            :class="
                                isExpanded
                                    ? 'max-w-full opacity-100'
                                    : 'max-w-0 opacity-0'
                            "
                        >
                            {{ item.label }}
                        </span>
                    </button>
                </div>
            </nav>

            <div class="pt-4">
                <button
                    class="flex items-center w-full p-3 rounded-lg transition-colors duration-200 group hover:bg-gray-100 text-gray-600"
                    @click="$router.push('/settings')"
                >
                    <SettingsIcon class="w-6 h-6 flex-shrink-0" />
                    <span
                        class="ml-3 overflow-hidden transition-all duration-300 whitespace-nowrap"
                        :class="
                            isExpanded
                                ? 'max-w-full opacity-100'
                                : 'max-w-0 opacity-0'
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

defineProps({
    activeTab: {
        type: String,
        default: "home",
    },
});

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
