<template>
    <div class="min-h-screen bg-gray-50 py-8">
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <!-- Header -->
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900">Token Manager</h1>
                <p class="mt-2 text-gray-600">
                    Manage authentication tokens and user access
                </p>
            </div>

            <!-- Create Token Section -->
            <div class="bg-white shadow-sm rounded-lg p-6 mb-8">
                <h2 class="text-xl font-semibold text-gray-900 mb-4">
                    Create New Token
                </h2>
                <form @submit.prevent="createToken" class="space-y-4">
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div>
                            <label
                                for="kind"
                                class="block text-sm font-medium text-gray-700 mb-1"
                            >
                                Token Kind
                            </label>
                            <select
                                id="kind"
                                v-model="newToken.kind.kind"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                required
                            >
                                <option value="">Select Kind</option>
                                <option value="NotVerified">
                                    Not Verified
                                </option>
                                <option value="Admin">Admin</option>
                                <option value="User">User</option>
                                <option value="Manager">Manager</option>
                            </select>
                        </div>
                        <div>
                            <label
                                for="single"
                                class="block text-sm font-medium text-gray-700 mb-1"
                            >
                                Single Use
                            </label>
                            <select
                                id="single"
                                v-model="newToken.kind.single"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            >
                                <option :value="true">Yes</option>
                                <option :value="false">No</option>
                            </select>
                        </div>
                        <div>
                            <label
                                for="userId"
                                class="block text-sm font-medium text-gray-700 mb-1"
                            >
                                User Email (Optional)
                            </label>
                            <input
                                id="userId"
                                v-model="newToken.user_id"
                                type="email"
                                placeholder="user@example.com"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            />
                        </div>
                    </div>
                    <div class="flex justify-end">
                        <button
                            type="submit"
                            :disabled="loading || !newToken.kind.kind"
                            class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            <PlusIcon class="w-4 h-4 mr-2" />
                            {{ loading ? "Creating..." : "Create Token" }}
                        </button>
                    </div>
                </form>
            </div>

            <!-- Tokens List -->
            <div class="bg-white shadow-sm rounded-lg overflow-hidden">
                <div class="px-6 py-4 border-b border-gray-200">
                    <h2 class="text-xl font-semibold text-gray-900">
                        Active Tokens
                    </h2>
                </div>

                <div
                    v-if="loading && tokens.length === 0"
                    class="p-8 text-center"
                >
                    <div
                        class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"
                    ></div>
                    <p class="mt-2 text-gray-500">Loading tokens...</p>
                </div>

                <div v-else-if="tokens.length === 0" class="p-8 text-center">
                    <div class="text-gray-400">
                        <DocumentIcon class="w-12 h-12 mx-auto mb-4" />
                        <p class="text-lg font-medium">No tokens found</p>
                        <p class="text-sm">
                            Create your first token to get started
                        </p>
                    </div>
                </div>

                <div v-else class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead class="bg-gray-50">
                            <tr>
                                <th
                                    class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                                >
                                    Token ID
                                </th>
                                <th
                                    class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                                >
                                    Kind
                                </th>
                                <th
                                    class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                                >
                                    Single Use
                                </th>
                                <th
                                    class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                                >
                                    User Email
                                </th>
                                <th
                                    class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider"
                                >
                                    Actions
                                </th>
                            </tr>
                        </thead>
                        <tbody class="bg-white divide-y divide-gray-200">
                            <tr
                                v-for="token in tokens"
                                :key="token.token_id"
                                class="hover:bg-gray-50"
                            >
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <div class="flex items-center">
                                        <code
                                            class="text-sm font-mono text-gray-900 bg-gray-100 px-2 py-1 rounded"
                                        >
                                            {{ token.token_id }}
                                        </code>
                                        <button
                                            @click="
                                                copyToClipboard(token.token_id)
                                            "
                                            class="ml-2 p-1 text-gray-400 hover:text-gray-600"
                                            title="Copy token ID"
                                        >
                                            <ClipboardIcon class="w-4 h-4" />
                                        </button>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <span
                                        :class="
                                            getKindBadgeClass(token.kind.kind)
                                        "
                                        class="inline-flex px-2 py-1 text-xs font-semibold rounded-full"
                                    >
                                        {{ token.kind.kind }}
                                    </span>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <span
                                        :class="
                                            token.kind.single
                                                ? 'text-orange-600 bg-orange-100'
                                                : 'text-green-600 bg-green-100'
                                        "
                                        class="inline-flex px-2 py-1 text-xs font-semibold rounded-full"
                                    >
                                        {{
                                            token.kind.single
                                                ? "Single Use"
                                                : "Multi Use"
                                        }}
                                    </span>
                                </td>
                                <td
                                    class="px-6 py-4 whitespace-nowrap text-sm text-gray-900"
                                >
                                    {{ token.user_id || "Not specified" }}
                                </td>
                                <td
                                    class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium"
                                >
                                    <button
                                        @click="deleteToken(token.token_id)"
                                        :disabled="loading"
                                        class="inline-flex items-center px-3 py-1 border border-transparent text-sm leading-4 font-medium rounded-md text-red-700 bg-red-100 hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 disabled:opacity-50 disabled:cursor-not-allowed"
                                    >
                                        <TrashIcon class="w-4 h-4 mr-1" />
                                        Delete
                                    </button>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <!-- Success/Error Messages -->
            <div
                v-if="message"
                :class="messageClass"
                class="fixed top-4 right-4 px-4 py-3 rounded-md shadow-lg z-50"
            >
                <div class="flex items-center">
                    <CheckCircleIcon
                        v-if="messageType === 'success'"
                        class="w-5 h-5 mr-2"
                    />
                    <ExclamationCircleIcon v-else class="w-5 h-5 mr-2" />
                    <span>{{ message }}</span>
                    <button
                        @click="message = ''"
                        class="ml-4 text-current opacity-75 hover:opacity-100"
                    >
                        <XMarkIcon class="w-4 h-4" />
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup>
import { ref, onMounted, computed } from "vue";

// Icons (you may need to install @heroicons/vue or use your preferred icon library)
const PlusIcon = defineComponent({
    template:
        '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>',
});

const TrashIcon = defineComponent({
    template:
        '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path></svg>',
});

const ClipboardIcon = defineComponent({
    template:
        '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"></path></svg>',
});

const DocumentIcon = defineComponent({
    template:
        '<svg class="w-12 h-12" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path></svg>',
});

const CheckCircleIcon = defineComponent({
    template:
        '<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>',
});

const ExclamationCircleIcon = defineComponent({
    template:
        '<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>',
});

const XMarkIcon = defineComponent({
    template:
        '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>',
});

// Reactive data
const tokens = ref([]);
const loading = ref(false);
const message = ref("");
const messageType = ref("success");

const newToken = ref({
    kind: {
        kind: "",
        single: true,
    },
    user_id: "",
});

// Computed properties
const messageClass = computed(() => {
    return messageType.value === "success"
        ? "bg-green-100 border border-green-400 text-green-700"
        : "bg-red-100 border border-red-400 text-red-700";
});

// Methods
const showMessage = (msg, type = "success") => {
    message.value = msg;
    messageType.value = type;
    setTimeout(() => {
        message.value = "";
    }, 3000);
};

const getKindBadgeClass = (kind) => {
    const classes = {
        NotVerified: "text-yellow-800 bg-yellow-100",
        Admin: "text-red-800 bg-red-100",
        User: "text-blue-800 bg-blue-100",
        Manager: "text-purple-800 bg-purple-100",
    };
    return classes[kind] || "text-gray-800 bg-gray-100";
};

const copyToClipboard = async (text) => {
    try {
        await navigator.clipboard.writeText(text);
        showMessage("Token ID copied to clipboard!", "success");
    } catch (err) {
        showMessage("Failed to copy to clipboard", "error");
    }
};

// Fake API functions (replace with real API calls)
const loadTokens = async () => {
    loading.value = true;
    try {
        // Simulate API call
        await new Promise((resolve) => setTimeout(resolve, 1000));

        // Mock data
        tokens.value = [
            {
                kind: { kind: "NotVerified", single: true },
                token_id: "tok_1234567890abcdef",
                user_id: "user@example.com",
            },
            {
                kind: { kind: "Admin", single: false },
                token_id: "tok_admin_xyz123",
                user_id: "admin@example.com",
            },
            {
                kind: { kind: "User", single: true },
                token_id: "tok_user_abc789",
                user_id: "",
            },
        ];
    } catch (error) {
        showMessage("Failed to load tokens", "error");
    } finally {
        loading.value = false;
    }
};

const createToken = async () => {
    if (!newToken.value.kind.kind) return;

    loading.value = true;
    try {
        // Simulate API call
        await new Promise((resolve) => setTimeout(resolve, 500));

        // Generate fake token ID
        const tokenId = `tok_${newToken.value.kind.kind.toLowerCase()}_${Math.random().toString(36).substr(2, 12)}`;

        const createdToken = {
            kind: {
                kind: newToken.value.kind.kind,
                single: newToken.value.kind.single,
            },
            token_id: tokenId,
            user_id: newToken.value.user_id || "",
        };

        tokens.value.unshift(createdToken);

        // Reset form
        newToken.value = {
            kind: {
                kind: "",
                single: true,
            },
            user_id: "",
        };

        showMessage("Token created successfully!", "success");
    } catch (error) {
        showMessage("Failed to create token", "error");
    } finally {
        loading.value = false;
    }
};

const deleteToken = async (tokenId) => {
    if (!confirm("Are you sure you want to delete this token?")) return;

    loading.value = true;
    try {
        // Simulate API call
        await new Promise((resolve) => setTimeout(resolve, 500));

        tokens.value = tokens.value.filter(
            (token) => token.token_id !== tokenId,
        );
        showMessage("Token deleted successfully!", "success");
    } catch (error) {
        showMessage("Failed to delete token", "error");
    } finally {
        loading.value = false;
    }
};

// Lifecycle
onMounted(() => {
    loadTokens();
});
</script>

<style scoped>
/* Additional custom styles if needed */
</style>
