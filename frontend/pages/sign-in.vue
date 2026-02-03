<template>
    <FloatingContainer>
        <SigninSignInWelcomeMessage message="Sign in to your account" />
        <div class="space-y-6">
            <InputFieldBase
                v-if="is_email"
                id="email"
                v-model="state.email"
                type="email"
                label="Email address"
                :disabled="loading"
                auto-complete="email"
                @click:label="() => (is_email = !is_email)"
            />
            <InputFieldBase
                v-else
                id="name"
                v-model="state.username"
                :disabled="loading"
                type="text"
                label="Name"
                auto-complete="email"
                @click:label="() => (is_email = !is_email)"
            />
            <InputFieldBase
                id="password"
                v-model="state.password"
                :disabled="loading"
                type="password"
                label="Password"
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
        <p class="mt-10 text-center text-sm/6 text-gray-500">
            No Account?
            <NuxtLink
                to="/sign-up"
                class="font-semibold text-indigo-600 hover:text-indigo-500"
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
import { validateUsername } from "~/utils/validation";
import FloatingContainer from "~/components/container/FloatingContainer.vue";
import PrimaryButton from "~/components/button/PrimaryButton.vue";
import InputFieldBase from "~/components/input/InputFieldBase.vue";

export default {
    components: {
        FloatingContainer,
        OtherSiteLogin,
        PrimaryButton,
        OrContinueWith,
        InputFieldBase,
        ForgotPassword,
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
                    // headers: {Authorization: `Bearer `}
                });
                await setTokens(resp.access_token, resp.refresh_token);
                navigateTo("/");
            } catch (e) {
                //TODO:
            }

            this.loading = false;
        },
    },
};

//const data2 = await $manRead('/api/v1/manga/home');
</script>
