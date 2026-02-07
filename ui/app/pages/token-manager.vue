<template>
    <div
        class="flex h-screen w-full bg-slate-50 text-slate-900 dark:bg-slate-950 dark:text-slate-100"
    >
        <div class="no-scrollbar h-screen flex-1 overflow-y-auto">
            <div
                class="flex min-h-full w-full flex-col gap-8 px-6 py-8 sm:px-8"
            >
                <div class="mb-2">
                    <h1
                        class="text-3xl font-bold text-slate-900 dark:text-slate-100"
                    >
                        Token Manager
                    </h1>
                    <p class="mt-2 text-slate-600 dark:text-slate-400">
                        Manage authentication tokens and user access
                    </p>
                </div>

                <div
                    class="mx-auto w-full max-w-5xl rounded-lg border border-slate-200 bg-white/80 p-8 shadow-sm dark:border-slate-800 dark:bg-slate-900/80 sm:p-10"
                >
                    <h2
                        class="mb-6 text-xl font-semibold text-slate-900 dark:text-slate-100"
                    >
                        Create New Token
                    </h2>
                    <form
                        class="mx-auto w-full max-w-4xl space-y-6"
                        @submit.prevent="createToken"
                    >
                        <div class="grid grid-cols-1 gap-6 md:grid-cols-3">
                            <div>
                                <label
                                    for="kind"
                                    class="mb-1 block text-sm font-medium text-slate-700 dark:text-slate-200"
                                >
                                    Token Kind
                                </label>
                                <select
                                    id="kind"
                                    v-model="newToken.kind.kind"
                                    class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-slate-900 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
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
                                    class="mb-1 block text-sm font-medium text-slate-700 dark:text-slate-200"
                                >
                                    Single Use
                                </label>
                                <select
                                    id="single"
                                    v-model="newToken.kind.single"
                                    class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-slate-900 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
                                >
                                    <option :value="true">Yes</option>
                                    <option :value="false">No</option>
                                </select>
                            </div>
                            <div>
                                <label
                                    for="userId"
                                    class="mb-1 block text-sm font-medium text-slate-700 dark:text-slate-200"
                                >
                                    User Email (Optional)
                                </label>
                                <input
                                    id="userId"
                                    v-model="newToken.user_id"
                                    type="email"
                                    placeholder="user@example.com"
                                    class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-slate-900 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
                                />
                            </div>
                        </div>
                        <div class="flex justify-end pt-2">
                            <button
                                type="submit"
                                :disabled="loading || !newToken.kind.kind"
                                class="inline-flex items-center rounded-md bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm transition hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:cursor-not-allowed disabled:opacity-50"
                            >
                                <PlusIcon class="w-4 h-4 mr-2" />
                                {{ loading ? "Creating..." : "Create Token" }}
                            </button>
                        </div>
                    </form>
                </div>

                <!-- Tokens List -->
                <div
                    class="flex-1 w-full overflow-hidden rounded-lg border border-slate-200 bg-white/80 shadow-sm dark:border-slate-800 dark:bg-slate-900/80"
                >
                    <div
                        class="border-b border-slate-200 px-8 py-5 dark:border-slate-800"
                    >
                        <h2
                            class="text-xl font-semibold text-slate-900 dark:text-slate-100"
                        >
                            Active Tokens
                        </h2>
                    </div>

                    <div
                        v-if="loading && tokens.length === 0"
                        class="p-8 text-center"
                    >
                        <div
                            class="mx-auto h-8 w-8 animate-spin rounded-full border-b-2 border-indigo-500"
                        />
                        <p class="mt-2 text-slate-500 dark:text-slate-400">
                            Loading tokens...
                        </p>
                    </div>

                    <div
                        v-else-if="tokens.length === 0"
                        class="p-8 text-center"
                    >
                        <div class="text-slate-400">
                            <DocumentIcon class="w-12 h-12 mx-auto mb-4" />
                            <p class="text-lg font-medium">No tokens found</p>
                            <p class="text-sm">
                                Create your first token to get started
                            </p>
                        </div>
                    </div>

                    <div v-else class="h-full w-full overflow-x-auto">
                        <table
                            class="w-full divide-y divide-slate-200 dark:divide-slate-800"
                        >
                            <thead class="bg-slate-50 dark:bg-slate-900/60">
                                <tr>
                                    <th
                                        class="px-8 py-3 text-left text-xs font-medium uppercase tracking-wider text-slate-500 dark:text-slate-400"
                                    >
                                        Token ID
                                    </th>
                                    <th
                                        class="px-8 py-3 text-left text-xs font-medium uppercase tracking-wider text-slate-500 dark:text-slate-400"
                                    >
                                        Kind
                                    </th>
                                    <th
                                        class="px-8 py-3 text-left text-xs font-medium uppercase tracking-wider text-slate-500 dark:text-slate-400"
                                    >
                                        Single Use
                                    </th>
                                    <th
                                        class="px-8 py-3 text-left text-xs font-medium uppercase tracking-wider text-slate-500 dark:text-slate-400"
                                    >
                                        User Email
                                    </th>
                                    <th
                                        class="px-8 py-3 text-right text-xs font-medium uppercase tracking-wider text-slate-500 dark:text-slate-400"
                                    >
                                        Actions
                                    </th>
                                </tr>
                            </thead>
                            <tbody
                                class="divide-y divide-slate-200 bg-white/80 dark:divide-slate-800 dark:bg-slate-900/60"
                            >
                                <tr
                                    v-for="token in tokens"
                                    :key="token.token_id"
                                    class="transition hover:bg-slate-50 dark:hover:bg-slate-900"
                                >
                                    <td
                                        class="px-8 py-4 whitespace-nowrap text-left"
                                    >
                                        <div class="flex items-center">
                                            <code
                                                class="rounded bg-slate-100 px-2 py-1 text-sm font-mono text-slate-900 dark:bg-slate-800 dark:text-slate-100"
                                            >
                                                {{ token.token_id }}
                                            </code>
                                            <button
                                                class="ml-2 p-1 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
                                                title="Copy token ID"
                                                @click="
                                                    copyToClipboard(
                                                        token.token_id,
                                                    )
                                                "
                                            >
                                                <ClipboardIcon
                                                    class="w-4 h-4"
                                                />
                                            </button>
                                        </div>
                                    </td>
                                    <td
                                        class="px-8 py-4 whitespace-nowrap text-left"
                                    >
                                        <span
                                            :class="
                                                getKindBadgeClass(
                                                    token.kind.kind,
                                                )
                                            "
                                            class="inline-flex px-2 py-1 text-xs font-semibold rounded-full"
                                        >
                                            {{ token.kind.kind }}
                                        </span>
                                    </td>
                                    <td
                                        class="px-8 py-4 whitespace-nowrap text-left"
                                    >
                                        <span
                                            :class="
                                                token.kind.single
                                                    ? 'text-amber-700 bg-amber-100 dark:text-amber-200 dark:bg-amber-500/20'
                                                    : 'text-emerald-700 bg-emerald-100 dark:text-emerald-200 dark:bg-emerald-500/20'
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
                                        class="px-8 py-4 whitespace-nowrap text-left text-sm text-slate-900 dark:text-slate-100"
                                    >
                                        {{ token.user_id || "Not specified" }}
                                    </td>
                                    <td
                                        class="px-8 py-4 whitespace-nowrap text-right text-sm font-medium"
                                    >
                                        <button
                                            class="inline-flex items-center rounded-md bg-rose-100 px-3 py-1 text-sm font-medium leading-4 text-rose-700 transition hover:bg-rose-200 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-rose-500/20 dark:text-rose-200 dark:hover:bg-rose-500/30"
                                            :disabled="loading"
                                            @click="deleteToken(token.token_id)"
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
                    class="fixed top-4 right-4 z-50 rounded-md px-4 py-3 shadow-lg"
                >
                    <div class="flex items-center">
                        <CheckCircleIcon
                            v-if="messageType === 'success'"
                            class="w-5 h-5 mr-2"
                        />
                        <ExclamationCircleIcon v-else class="w-5 h-5 mr-2" />
                        <span>{{ message }}</span>
                        <button
                            class="ml-4 text-current opacity-75 hover:opacity-100"
                            @click="message = ''"
                        >
                            <XMarkIcon class="w-4 h-4" />
                        </button>
                    </div>
                </div>
            </div>
        </div>
        <div class="hidden lg:block">
            <Sidebar active-tab="tokens" />
        </div>
    </div>
</template>

<script setup>
import { ref, onMounted, computed } from "vue";
import Sidebar from "~/components/sidebar/SideBar.vue";

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
        NotVerified:
            "text-amber-800 bg-amber-100 dark:text-amber-200 dark:bg-amber-500/20",
        Admin: "text-rose-800 bg-rose-100 dark:text-rose-200 dark:bg-rose-500/20",
        User: "text-sky-800 bg-sky-100 dark:text-sky-200 dark:bg-sky-500/20",
        Manager:
            "text-violet-800 bg-violet-100 dark:text-violet-200 dark:bg-violet-500/20",
    };
    return (
        classes[kind] ||
        "text-slate-800 bg-slate-100 dark:text-slate-200 dark:bg-slate-500/20"
    );
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
