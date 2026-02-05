<template>
    <FloatingContainer :small="false">
        <SignInWelcomeMessage message="Sign in to your account" />
        <div class="space-y-6">
            <InputFieldBase
                v-if="is_email"
                v-model="state.email"
                iid="email"
                type="email"
                label="Email address"
                :disabled="loading"
                auto-complete="email"
                :open-upwards="false"
                :open-button="false"
                @click:label="() => (is_email = !is_email)"
            />
            <InputFieldBase
                v-else
                v-model="state.username"
                iid="name"
                :disabled="loading"
                type="text"
                label="Name"
                auto-complete="email"
                :open-upwards="false"
                :open-button="false"
                @click:label="() => (is_email = !is_email)"
            />
            <InputFieldBase
                v-model="state.password"
                iid="password"
                :disabled="loading"
                type="password"
                label="Password"
                :open-upwards="false"
                :open-button="false"
                auto-complete="current-password"
            />
            <ForgotPassword name="Forgot password?" to="/forgot-password" />
            <PrimaryButton
                name="Sign in"
                :enabled="submit_available"
                @click="login"
            />
        </div>
        <div>
            <OrContinueWith />
            <OtherSiteLogin />
        </div>
        <p class="mt-10 text-center text-sm/6 text-slate-500 dark:text-slate-400">
            No Account?
            <NuxtLink
                to="/sign-up"
                class="font-semibold text-indigo-600 hover:text-indigo-500 dark:text-indigo-400 dark:hover:text-indigo-300"
                >Register here
            </NuxtLink>
        </p>
    </FloatingContainer>
</template>
<script lang="ts">
import OtherSiteLogin from "~/components/signin/OtherSiteLogin.vue";
import OrContinueWith from "~/components/signin/OrContinueWith.vue";
import ForgotPassword from "~/components/signin/ForgotPassword.vue";
import { setTokens } from "~/utils/auth";
import { navigateTo } from "#imports";
import {
    validateUsername,
    validateEmail,
    validatePassword,
} from "~/utils/validation";
import FloatingContainer from "~/components/container/FloatingContainer.vue";
import PrimaryButton from "~/components/button/PrimaryButton.vue";
import InputFieldBase from "~/components/input/InputFieldBase.vue";
import SignInWelcomeMessage from "~/components/signin/SignInWelcomeMessage.vue";

export default {
    components: {
        FloatingContainer,
        OtherSiteLogin,
        PrimaryButton,
        OrContinueWith,
        InputFieldBase,
        ForgotPassword,
        SignInWelcomeMessage,
    },
    setup() {
        useHead({
            title: "SignIn - ManRead",
        });
        return {
            is_email: ref(false),
            loading: ref(false),
            state: reactive({
                email: "",
                username: "",
                password: "",
            }),
        };
    },
    computed: {
        submit_available() {
            return (
                !this.loading &&
                (this.is_email
                    ? validateEmail(this.state.email)
                    : validateUsername(this.state.username) &&
                      validatePassword(this.state.password))
            );
        },
    },
    methods: {
        async login() {
            if (this.loading) {
                return;
            }
            this.loading = true;
            const { $manRead } = useNuxtApp();
            try {
                const resp = await $manRead("/api/v1/auth/sign-in", {
                    method: "POST",
                    body: this.is_email
                        ? {
                              email: this.state.email,
                              password: this.state.password,
                          }
                        : {
                              username: this.state.username,
                              password: this.state.password,
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
