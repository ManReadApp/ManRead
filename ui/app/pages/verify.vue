<template>
    <FloatingContainer :small="true">
        <SigninSignInWelcomeMessage message="Assign yourself a Role" />
        <div class="space-y-6">
            <div class="w-full flex">
                <OtpInput
                    ref="otpInput"
                    input-classes="w-9 h-9 rounded-md p-2 text-slate-900 ring-1 ring-slate-300/70 placeholder:text-slate-400 focus:ring-2 focus:ring-indigo-500 dark:bg-slate-900 dark:text-slate-100 dark:ring-slate-700"
                    separator=""
                    input-type="letter-numeric"
                    :num-inputs="6"
                    :should-auto-focus="true"
                    :should-focus-order="true"
                    :placeholder="[]"
                    @on-change="(v) => (token = v)"
                />
            </div>
            <PrimaryButton
                :enabled="is_submittable"
                name="Get Role"
                @click="submit"
            />
        </div>
    </FloatingContainer>
</template>
<script lang="ts">
import { setTokens } from "~/utils/auth";
import { navigateTo } from "#imports";
import OtpInput from "vue3-otp-input";
import FloatingContainer from "~/components/container/FloatingContainer.vue";
import PrimaryButton from "~/components/button/PrimaryButton.vue";

export default {
    components: { FloatingContainer, PrimaryButton, OtpInput },
    setup() {
        useHead({
            title: "Set Role - ManRead",
        });
        return { loading: ref(false), token: ref("") };
    },
    computed: {
        is_submittable() {
            return !this.loading && this.token.length == 6;
        },
    },
    methods: {
        async submit() {
            if (this.loading) {
                return;
            }
            this.loading = true;
            const { $manRead } = useNuxtApp();
            try {
                const resp = await $manRead("/api/v1/auth/verify-account", {
                    method: "PUT",
                    body: {
                        key: this.token,
                    },
                    headers: {
                        Authorization: `Bearer ${await getAccessToken()}`,
                    },
                });
                await setTokens(resp.access_token, resp.refresh_token);
                navigateTo("/");
            } catch (e) {
                console.error(e);
                //TODO:
            }
            this.loading = false;
        },
    },
};
</script>
<style>
.otp-input-container {
    width: 100%;
    justify-content: space-between;
}
</style>
