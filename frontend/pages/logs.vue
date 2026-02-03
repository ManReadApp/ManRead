<template>
    <div class="min-h-screen bg-gray-50 p-6">
        <div class="max-w-7xl mx-auto">
            <!-- Header -->
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900 mb-2">
                    System Logs
                </h1>
                <p class="text-gray-600">Monitor and manage application logs</p>
            </div>

            <!-- Controls -->
            <div
                class="bg-white rounded-lg shadow-sm border border-gray-200 p-6 mb-6"
            >
                <div
                    class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4"
                >
                    <div class="flex flex-col sm:flex-row gap-3">
                        <button
                            :disabled="loading"
                            class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                            @click="refreshLogs"
                        >
                            <svg
                                class="w-4 h-4 mr-2"
                                :class="{ 'animate-spin': loading }"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                                />
                            </svg>
                            {{ loading ? "Refreshing..." : "Refresh" }}
                        </button>

                        <button
                            :disabled="loading || logs.length === 0"
                            class="inline-flex items-center px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                            @click="showClearModal = true"
                        >
                            <svg
                                class="w-4 h-4 mr-2"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                                />
                            </svg>
                            Clear Logs
                        </button>
                    </div>

                    <!-- Filter Controls -->
                    <div class="flex flex-col sm:flex-row gap-3">
                        <select
                            v-model="selectedLevel"
                            class="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                        >
                            <option value="">All Levels</option>
                            <option value="error">Error</option>
                            <option value="warn">Warning</option>
                            <option value="info">Info</option>
                            <option value="debug">Debug</option>
                        </select>

                        <div class="relative">
                            <!-- eslint-disable-next-line vue/html-self-closing -->
                            <input
                                v-model="searchQuery"
                                type="text"
                                placeholder="Search logs..."
                                class="pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 w-64"
                            />
                            <svg
                                class="w-4 h-4 text-gray-400 absolute left-3 top-3"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                                />
                            </svg>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Stats -->
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
                <div
                    class="bg-white p-6 rounded-lg shadow-sm border border-gray-200"
                >
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div
                                class="w-8 h-8 bg-blue-100 rounded-lg flex items-center justify-center"
                            >
                                <svg
                                    class="w-4 h-4 text-blue-600"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                                    />
                                </svg>
                            </div>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">
                                Total Logs
                            </p>
                            <p class="text-2xl font-bold text-gray-900">
                                {{ logs.length }}
                            </p>
                        </div>
                    </div>
                </div>

                <div
                    class="bg-white p-6 rounded-lg shadow-sm border border-gray-200"
                >
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div
                                class="w-8 h-8 bg-red-100 rounded-lg flex items-center justify-center"
                            >
                                <svg
                                    class="w-4 h-4 text-red-600"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                    />
                                </svg>
                            </div>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">
                                Errors
                            </p>
                            <p class="text-2xl font-bold text-gray-900">
                                {{ getLogCountByLevel("error") }}
                            </p>
                        </div>
                    </div>
                </div>

                <div
                    class="bg-white p-6 rounded-lg shadow-sm border border-gray-200"
                >
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div
                                class="w-8 h-8 bg-yellow-100 rounded-lg flex items-center justify-center"
                            >
                                <svg
                                    class="w-4 h-4 text-yellow-600"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.99-.833-2.76 0L3.054 16.5C2.284 18.167 3.246 20 4.786 20z"
                                    />
                                </svg>
                            </div>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">
                                Warnings
                            </p>
                            <p class="text-2xl font-bold text-gray-900">
                                {{ getLogCountByLevel("warn") }}
                            </p>
                        </div>
                    </div>
                </div>

                <div
                    class="bg-white p-6 rounded-lg shadow-sm border border-gray-200"
                >
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div
                                class="w-8 h-8 bg-green-100 rounded-lg flex items-center justify-center"
                            >
                                <svg
                                    class="w-4 h-4 text-green-600"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                    />
                                </svg>
                            </div>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">
                                Info
                            </p>
                            <p class="text-2xl font-bold text-gray-900">
                                {{ getLogCountByLevel("info") }}
                            </p>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Logs List -->
            <div class="bg-white rounded-lg shadow-sm border border-gray-200">
                <div class="px-6 py-4 border-b border-gray-200">
                    <h2 class="text-lg font-semibold text-gray-900">
                        Log Entries
                    </h2>
                </div>

                <div class="divide-y divide-gray-200 max-h-96 overflow-y-auto">
                    <div v-if="loading" class="p-6 text-center">
                        <div
                            class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"
                        />
                        <p class="mt-2 text-gray-500">Loading logs...</p>
                    </div>

                    <div
                        v-else-if="filteredLogs.length === 0"
                        class="p-6 text-center"
                    >
                        <svg
                            class="w-12 h-12 text-gray-400 mx-auto mb-4"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                            />
                        </svg>
                        <p class="text-gray-500">No logs found</p>
                    </div>

                    <div
                        v-for="log in filteredLogs"
                        :key="log.id"
                        class="p-4 hover:bg-gray-50 transition-colors"
                    >
                        <div class="flex items-start justify-between">
                            <div class="flex-1">
                                <div class="flex items-center mb-2">
                                    <span
                                        :class="getLevelBadgeClass(log.level)"
                                        class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium mr-3"
                                    >
                                        {{ log.level.toUpperCase() }}
                                    </span>
                                    <span class="text-sm text-gray-500">{{
                                        formatTimestamp(log.timestamp)
                                    }}</span>
                                </div>
                                <p class="text-gray-900 mb-1">
                                    {{ log.message }}
                                </p>
                                <div
                                    v-if="log.details"
                                    class="text-sm text-gray-600"
                                >
                                    <details class="mt-2">
                                        <summary
                                            class="cursor-pointer text-blue-600 hover:text-blue-700"
                                        >
                                            View Details
                                        </summary>
                                        <pre
                                            class="mt-2 bg-gray-100 p-3 rounded text-xs overflow-x-auto"
                                            >{{ log.details }}</pre
                                        >
                                    </details>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Clear Confirmation Modal -->
            <div
                v-if="showClearModal"
                class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
            >
                <div class="bg-white rounded-lg p-6 max-w-md w-full mx-4">
                    <h3 class="text-lg font-semibold text-gray-900 mb-4">
                        Clear All Logs
                    </h3>
                    <p class="text-gray-600 mb-6">
                        Are you sure you want to clear all logs? This action
                        cannot be undone.
                    </p>
                    <div class="flex justify-end space-x-3">
                        <button
                            class="px-4 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
                            @click="showClearModal = false"
                        >
                            Cancel
                        </button>
                        <button
                            :disabled="clearing"
                            class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                            @click="clearLogs"
                        >
                            {{ clearing ? "Clearing..." : "Clear All" }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";

// Reactive state
const logs = ref([]);
const loading = ref(false);
const clearing = ref(false);
const showClearModal = ref(false);
const selectedLevel = ref("");
const searchQuery = ref("");

// Mock data for demonstration
const mockLogs = [
    {
        id: 1,
        level: "info",
        message: "Application started successfully",
        timestamp: new Date(Date.now() - 1000 * 60 * 5),
        details: "Server listening on port 3000\nEnvironment: development",
    },
    {
        id: 2,
        level: "warn",
        message: "Database connection slow",
        timestamp: new Date(Date.now() - 1000 * 60 * 3),
        details: "Connection took 2.5 seconds to establish",
    },
    {
        id: 3,
        level: "error",
        message: "Failed to process user request",
        timestamp: new Date(Date.now() - 1000 * 60 * 2),
        details:
            "Error: Cannot read property 'id' of undefined\n  at /app/routes/users.js:42:15",
    },
    {
        id: 4,
        level: "debug",
        message: "Cache miss for user:123",
        timestamp: new Date(Date.now() - 1000 * 60 * 1),
        details: "Fetching user data from database",
    },
    {
        id: 5,
        level: "info",
        message: "User authentication successful",
        timestamp: new Date(),
        details: "User ID: 123, Session: abc-def-ghi",
    },
];

// Computed properties
const filteredLogs = computed(() => {
    let filtered = logs.value;

    // Filter by level
    if (selectedLevel.value) {
        filtered = filtered.filter((log) => log.level === selectedLevel.value);
    }

    // Filter by search query
    if (searchQuery.value) {
        const query = searchQuery.value.toLowerCase();
        filtered = filtered.filter(
            (log) =>
                log.message.toLowerCase().includes(query) ||
                (log.details && log.details.toLowerCase().includes(query)),
        );
    }

    // Sort by timestamp (newest first)
    return filtered.sort(
        (a, b) => new Date(b.timestamp) - new Date(a.timestamp),
    );
});

// Methods
const listLogs = async () => {
    loading.value = true;
    try {
        // TODO: Replace with actual API call
        await new Promise((resolve) => setTimeout(resolve, 1000));
        logs.value = [...mockLogs];
    } catch (error) {
        console.error("Failed to fetch logs:", error);
        // TODO: Show error notification
    } finally {
        loading.value = false;
    }
};

const clearLogs = async () => {
    clearing.value = true;
    try {
        // TODO: Replace with actual API call
        await new Promise((resolve) => setTimeout(resolve, 500));
        logs.value = [];
        showClearModal.value = false;
    } catch (error) {
        console.error("Failed to clear logs:", error);
        // TODO: Show error notification
    } finally {
        clearing.value = false;
    }
};

const refreshLogs = () => {
    listLogs();
};

const getLevelBadgeClass = (level) => {
    const classes = {
        error: "bg-red-100 text-red-800",
        warn: "bg-yellow-100 text-yellow-800",
        info: "bg-blue-100 text-blue-800",
        debug: "bg-gray-100 text-gray-800",
    };
    return classes[level] || "bg-gray-100 text-gray-800";
};

const getLogCountByLevel = (level) => {
    return logs.value.filter((log) => log.level === level).length;
};

const formatTimestamp = (timestamp) => {
    return new Date(timestamp).toLocaleString();
};

// Lifecycle
onMounted(() => {
    listLogs();
});
</script>
