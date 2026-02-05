<template>
    <slot v-if="state == 'Access'" />
    <div v-else-if="state == 'Loading'">Loading...</div>
    <div v-else-if="state == 'NoAccess'">Access denied</div>
    <div v-else-if="state == 'NoAccessInvalid'">
        Invalid access token <NuxtLink to="/sign-in">Back To Login</NuxtLink>
    </div>
    <div v-else-if="state == 'GotoVerify'">
        <NuxtLink to="/verify">Verify your email</NuxtLink>
    </div>
</template>

<script setup lang="ts">
import { getAccessToken, generateClaim } from "~/utils/auth.js";

const props = defineProps<{
    roles?: string[];
}>();

const { data: claim } = useAsyncData("claim", async () => {
    const at = await getAccessToken();
    if (!at) return -1;
    return generateClaim(at) ?? -1;
});

const state = computed(() => {
    const roles = props.roles ?? [];
    if (claim.value == -1) {
        return "NoAccessInvalid";
    } else if (claim.value == null) {
        return "Loading";
    } else {
        if (claim.value.role === "NotVerified") {
            return "GotoVerify";
        } else {
            if (roles.length === 0 || roles.includes(claim.value.role)) {
                return "Access";
            } else {
                return "NoAccess";
            }
        }
    }
});
</script>
