<template>
    <div class="h-screen bg-gray-100">
        <div class="bg-white shadow-sm border-b border-gray-200">
            <div class="overflow-x-auto">
                <div class="flex space-x-0 min-w-max">
                    <button
                        v-for="tab in tabs"
                        :key="`${tab.id.manga}-${tab.id.version}`"
                        :class="[
                            'px-4 py-3 text-sm font-medium border-b-2 transition-all duration-200 whitespace-nowrap',
                            activeTab?.manga === tab.id.manga &&
                            activeTab?.version === tab.id.version
                                ? 'text-blue-600 border-blue-500 bg-blue-50/50'
                                : 'text-gray-600 border-transparent hover:text-gray-900 hover:border-gray-300',
                        ]"
                        @click="switchTab(tab.id)"
                    >
                        {{ tab.name }}
                    </button>
                </div>
            </div>
        </div>

        <div class="p-1">
            <div class="bg-white rounded-xl shadow-sm border border-gray-200">
                <div class="px-6 py-4 border-b border-gray-200 bg-gray-50/50">
                    <div class="flex items-center justify-between">
                        <div class="flex items-center space-x-6">
                            <div class="relative">
                                <div class="relative">
                                    <button
                                        class="flex items-center justify-between w-48 px-3 py-2 text-sm bg-white border border-gray-300 rounded-lg hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                        @click="
                                            showStateDropdown =
                                                !showStateDropdown
                                        "
                                    >
                                        <span class="text-gray-700">
                                            {{
                                                visibleStates.length ===
                                                states.length
                                                    ? "All States"
                                                    : visibleStates.length === 1
                                                      ? visibleStates[0]
                                                      : `${visibleStates.length} selected`
                                            }}
                                        </span>
                                        <svg
                                            class="w-4 h-4 text-gray-400"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M19 9l-7 7-7-7"
                                            />
                                        </svg>
                                    </button>
                                    <div
                                        v-if="showStateDropdown"
                                        class="absolute z-10 w-full mt-1 bg-white border border-gray-300 rounded-lg shadow-lg"
                                    >
                                        <div class="p-2">
                                            <label
                                                v-for="state in states"
                                                :key="state"
                                                class="flex items-center space-x-2 p-2 hover:bg-gray-50 rounded cursor-pointer"
                                            >
                                                <!-- eslint-disable-next-line vue/html-self-closing -->
                                                <input
                                                    v-model="visibleStates"
                                                    type="checkbox"
                                                    :value="state"
                                                    class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                />
                                                <span
                                                    class="text-sm text-gray-700"
                                                    >{{ state }}</span
                                                >
                                            </label>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="flex items-center space-x-4">
                            <!-- Bulk Action Buttons -->
                            <div
                                v-if="selectedItems.length > 0"
                                class="flex items-center space-x-2"
                            >
                                <button
                                    class="px-3 py-2 text-sm font-medium bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors duration-200"
                                    title="Increment chapter numbers for selected items"
                                    @click="incrementChapter"
                                >
                                    Ch +
                                </button>
                                <button
                                    class="px-3 py-2 text-sm font-medium bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors duration-200"
                                    title="Decrement chapter numbers for selected items"
                                    @click="decrementChapter"
                                >
                                    Ch -
                                </button>
                                <button
                                    class="px-3 py-2 text-sm font-medium bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors duration-200"
                                    title="Set selected items to Approved"
                                    @click="setSelectedState('Approved')"
                                >
                                    Approve
                                </button>
                                <button
                                    class="px-3 py-2 text-sm font-medium bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors duration-200"
                                    title="Set selected items to Declined"
                                    @click="setSelectedState('Declined')"
                                >
                                    Decline
                                </button>
                                <span class="text-sm text-gray-600">
                                    ({{ selectedItems.length }} selected)
                                </span>
                            </div>

                            <button
                                :disabled="isSubmitting || !hasChanges"
                                class="px-6 py-2.5 text-sm font-medium bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 shadow-sm"
                                @click="submitChanges"
                            >
                                {{
                                    isSubmitting
                                        ? "Submitting..."
                                        : "Submit Changes"
                                }}
                            </button>
                        </div>
                    </div>
                </div>

                <div class="overflow-auto max-h-[calc(100vh-137px)]">
                    <table class="w-full">
                        <thead
                            class="bg-gray-50 sticky top-0 border-b border-gray-200 z-10"
                        >
                            <tr>
                                <th class="px-6 py-4 w-12">
                                    <!-- eslint-disable-next-line vue/html-self-closing -->
                                    <input
                                        type="checkbox"
                                        :checked="isAllSelected"
                                        class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                        @change="toggleSelectAll"
                                    />
                                </th>
                                <th class="px-6 py-4 text-left w-24">
                                    <button
                                        class="flex items-center space-x-2 text-xs font-semibold text-gray-600 uppercase tracking-wider hover:text-gray-900 transition-colors"
                                        @click="toggleSort"
                                    >
                                        <span>Chapter</span>
                                        <svg
                                            v-if="sortOrder === 'asc'"
                                            class="w-4 h-4"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M5 15l7-7 7 7"
                                            />
                                        </svg>
                                        <svg
                                            v-else
                                            class="w-4 h-4"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M19 9l-7 7-7-7"
                                            />
                                        </svg>
                                    </button>
                                </th>
                                <th
                                    class="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider"
                                >
                                    Name
                                </th>
                                <th
                                    class="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider w-24"
                                >
                                    Link
                                </th>
                                <th
                                    class="px-6 py-4 text-right text-xs font-semibold text-gray-600 uppercase tracking-wider w-32"
                                >
                                    State
                                </th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-200 bg-white">
                            <tr
                                v-for="item in filteredAndSortedItems"
                                :key="item.id"
                                :class="[
                                    'hover:bg-gray-50 transition-colors duration-150 border-b border-gray-100',
                                    ['Approved', 'Processed'].includes(
                                        item.state,
                                    )
                                        ? 'bg-gray-50/30'
                                        : '',
                                ]"
                            >
                                <td class="px-6 py-4">
                                    <!-- eslint-disable-next-line vue/html-self-closing -->
                                    <input
                                        v-if="
                                            !['Approved', 'Processed'].includes(
                                                item.state,
                                            )
                                        "
                                        v-model="selectedItems"
                                        type="checkbox"
                                        :value="item.id"
                                        class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                    />
                                </td>
                                <td class="w-24">
                                    <!-- eslint-disable-next-line vue/html-self-closing -->
                                    <input
                                        v-model="item.chapter"
                                        :disabled="
                                            ['Approved', 'Processed'].includes(
                                                item.state,
                                            )
                                        "
                                        :class="[
                                            'w-full h-full px-6 py-4 text-sm bg-transparent transition-all duration-200 outline-none focus:outline-none rounded-none',
                                            ['Approved', 'Processed'].includes(
                                                item.state,
                                            )
                                                ? 'text-gray-500 cursor-not-allowed'
                                                : 'hover:bg-gray-50 focus:bg-blue-50',
                                        ]"
                                        type="text"
                                        @input="markAsChanged(item.id)"
                                    />
                                </td>
                                <td class="">
                                    <!-- eslint-disable-next-line vue/html-self-closing -->
                                    <input
                                        v-model="item.name"
                                        :disabled="
                                            ['Approved', 'Processed'].includes(
                                                item.state,
                                            )
                                        "
                                        :class="[
                                            'w-full h-full px-6 py-4 text-sm bg-transparent transition-all duration-200 outline-none focus:outline-none rounded-none',
                                            ['Approved', 'Processed'].includes(
                                                item.state,
                                            )
                                                ? 'text-gray-500 cursor-not-allowed'
                                                : 'hover:bg-gray-50 focus:bg-blue-50',
                                        ]"
                                        @input="markAsChanged(item.id)"
                                    />
                                </td>
                                <td class="px-6 py-4 w-24">
                                    <a
                                        v-if="item.link"
                                        :href="item.link"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="inline-flex items-center justify-center w-8 h-8 text-blue-600 hover:text-blue-800 hover:bg-blue-50 rounded-full transition-colors duration-200"
                                        title="Open link"
                                    >
                                        <svg
                                            class="w-4 h-4"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                                            />
                                        </svg>
                                    </a>
                                </td>
                                <td class="px-6 py-4 text-right w-32">
                                    <div
                                        v-if="
                                            !['Approved', 'Processed'].includes(
                                                item.state,
                                            )
                                        "
                                        class="relative inline-flex items-center justify-end"
                                    >
                                        <select
                                            v-model="item.state"
                                            :class="[
                                                'text-sm bg-transparent transition-all duration-200 outline-none focus:outline-none appearance-none cursor-pointer text-right pl-3 font-bold rounded-none',
                                                'hover:bg-gray-50 focus:bg-blue-50',
                                                getStateColor(item.state),
                                            ]"
                                            @change="markAsChanged(item.id)"
                                        >
                                            <option value="Approved">
                                                Approved
                                            </option>
                                            <option value="Declined">
                                                Declined
                                            </option>
                                            <option value="Pending" disabled>
                                                Pending
                                            </option>
                                            <option value="Processed" disabled>
                                                Processed
                                            </option>
                                        </select>
                                        <svg
                                            class="absolute left-0 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400 pointer-events-none"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M19 9l-7 7-7-7"
                                            />
                                        </svg>
                                    </div>
                                    <span
                                        v-else
                                        :class="[
                                            'text-sm font-bold',
                                            getStateColor(item.state),
                                        ]"
                                    >
                                        {{ item.state }}
                                    </span>
                                </td>
                            </tr>
                        </tbody>
                    </table>

                    <div
                        v-if="filteredAndSortedItems.length === 0"
                        class="text-center py-16 text-gray-500"
                    >
                        <div class="text-lg font-medium">No items found</div>
                        <div class="text-sm mt-2">
                            Try adjusting your filters or switch to a different
                            segment
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div
            v-if="isLoading"
            class="fixed inset-0 bg-black/20 backdrop-blur-sm flex items-center justify-center z-50"
        >
            <div class="bg-white rounded-xl p-6 shadow-xl">
                <div class="flex items-center space-x-3">
                    <div
                        class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"
                    />
                    <span class="text-gray-700 font-medium"
                        >Loading data...</span
                    >
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { getAccessToken } from "~/utils/auth";
import { getTitle } from "~/utils/titles";

const activeTab = ref(null);
const items = ref([]);
const selectedItems = ref([]);
const visibleStates = ref(["Approved", "Pending", "Declined", "Processed"]);
const sortOrder = ref("asc");
const changedItems = ref(new Set());
const isSubmitting = ref(false);
const isLoading = ref(false);
const showStateDropdown = ref(false);

const states = ["Approved", "Pending", "Declined", "Processed"];
const tabs = ref([]);

const generateTabs = async () => {
    const { $manRead } = useNuxtApp();
    const data = await $manRead("/api/v1/external/mangas", {
        method: "POST",
        body: { limit: 99999, page: 1 },
        headers: {
            Authorization: `Bearer ${await getAccessToken()}`,
        },
    });
    return data.map((item) => {
        return {
            name: getTitle(item.names) + " - " + item.version,
            id: { manga: item.manga_id, version: item.version_id },
        };
    });
};

const filteredAndSortedItems = computed(() => {
    let filtered = items.value.filter((item) => {
        return visibleStates.value.includes(item.state);
    });

    return filtered.sort((a, b) => {
        const aChapter = parseFloat(a.chapter) || 0;
        const bChapter = parseFloat(b.chapter) || 0;
        return sortOrder.value === "asc"
            ? aChapter - bChapter
            : bChapter - aChapter;
    });
});

const isAllSelected = computed(() => {
    const selectableItems = filteredAndSortedItems.value.filter(
        (item) => !["Approved", "Processed"].includes(item.state),
    );
    return (
        selectableItems.length > 0 &&
        selectableItems.every((item) => selectedItems.value.includes(item.id))
    );
});

const hasChanges = computed(() => {
    return changedItems.value.size > 0;
});

const getStateColor = (state) => {
    const colors = {
        Approved: "text-green-700",
        Pending: "text-yellow-700",
        Declined: "text-red-700",
        Processed: "text-blue-700",
    };
    return colors[state] || "text-gray-700";
};

const toggleSelectAll = () => {
    const selectableItems = filteredAndSortedItems.value.filter(
        (item) => !["Approved", "Processed"].includes(item.state),
    );

    if (isAllSelected.value) {
        selectedItems.value = selectedItems.value.filter(
            (id) => !selectableItems.some((item) => item.id === id),
        );
    } else {
        const newSelections = selectableItems
            .filter((item) => !selectedItems.value.includes(item.id))
            .map((item) => item.id);
        selectedItems.value.push(...newSelections);
    }
};

const toggleSort = () => {
    sortOrder.value = sortOrder.value === "asc" ? "desc" : "asc";
};

const markAsChanged = (itemId) => {
    changedItems.value.add(itemId);
};

const switchTab = async (tabId) => {
    activeTab.value = tabId;
    await generateData();
};

const generateData = async () => {
    isLoading.value = true;
    const { $manRead } = useNuxtApp();

    if (!activeTab.value) {
        isLoading.value = false;
        return;
    }

    const data = await $manRead("/api/v1/external/chapters/list", {
        method: "POST",
        body: {
            manga_id: activeTab.value!.manga,
            version_id: activeTab.value!.version,
        },
        headers: {
            Authorization: `Bearer ${await getAccessToken()}`,
        },
    });
    items.value = data;
    selectedItems.value = [];
    changedItems.value.clear();
    isLoading.value = false;
};

const submitChanges = async () => {
    isSubmitting.value = true;

    await new Promise((resolve) => setTimeout(resolve, 1500));

    console.log(
        "Submitting changes for items:",
        Array.from(changedItems.value),
    );
    console.log(
        "Changed items data:",
        items.value.filter((item) => changedItems.value.has(item.id)),
    );

    changedItems.value.clear();
    isSubmitting.value = false;
};

const incrementChapter = () => {
    const selectedItemObjects = items.value.filter(
        (item) =>
            selectedItems.value.includes(item.id) &&
            !["Approved", "Processed"].includes(item.state),
    );

    selectedItemObjects.forEach((item) => {
        const currentChapter = parseFloat(item.chapter) || 0;
        const newChapter = currentChapter + 1;
        item.chapter =
            newChapter % 1 === 0
                ? newChapter.toString()
                : newChapter.toFixed(1);
        markAsChanged(item.id);
    });
};

const decrementChapter = () => {
    const selectedItemObjects = items.value.filter(
        (item) =>
            selectedItems.value.includes(item.id) &&
            !["Approved", "Processed"].includes(item.state),
    );

    selectedItemObjects.forEach((item) => {
        const currentChapter = parseFloat(item.chapter) || 0;
        const newChapter = Math.max(1, currentChapter - 1);
        item.chapter =
            newChapter % 1 === 0
                ? newChapter.toString()
                : newChapter.toFixed(1);
        markAsChanged(item.id);
    });
};

const setSelectedState = (state) => {
    const selectedItemObjects = items.value.filter(
        (item) =>
            selectedItems.value.includes(item.id) &&
            !["Approved", "Processed"].includes(item.state),
    );

    selectedItemObjects.forEach((item) => {
        item.state = state;
        markAsChanged(item.id);
    });

    // Clear selection after setting state to prevent further bulk actions on approved/processed items
    selectedItems.value = selectedItems.value.filter((id) => {
        const item = items.value.find((i) => i.id === id);
        return item && !["Approved", "Processed"].includes(item.state);
    });
};

const handleClickOutside = (event) => {
    if (!event.target.closest(".relative")) {
        showStateDropdown.value = false;
    }
};

onMounted(async () => {
    document.addEventListener("click", handleClickOutside);

    // Generate tabs first
    const generatedTabs = await generateTabs();
    tabs.value = generatedTabs;

    // Set the first tab as active if available
    if (generatedTabs.length > 0) {
        activeTab.value = generatedTabs[0].id;
    }

    await generateData();
});

onUnmounted(() => {
    document.removeEventListener("click", handleClickOutside);
});
</script>

<style scoped>
/* Custom scrollbar for webkit browsers */
::-webkit-scrollbar {
    width: 8px;
    height: 8px;
}

::-webkit-scrollbar-track {
    background: #f1f5f9;
    border-radius: 4px;
}

::-webkit-scrollbar-thumb {
    background: #cbd5e1;
    border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
    background: #94a3b8;
}
</style>
