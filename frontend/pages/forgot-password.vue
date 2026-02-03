<script lang="ts">
import InputFieldBase from "~/components/input/InputFieldBase.vue";

import ForgotPassword from "~/components/signin/ForgotPassword.vue";
import Warn from "~/components/hints/Warn.vue";
import FloatingContainer from "~/components/container/FloatingContainer.vue";
import PrimaryButton from "~/components/button/PrimaryButton.vue";

export default {
    components: {
        Warn,
        ForgotPassword,
        InputFieldBase,
        FloatingContainer,
        PrimaryButton,
    },
    setup() {
        useHead({
            title: "Forgot Password - ManRead",
        });
        return {
            is_email: ref(true),
            email: ref(""),
            done: ref(false),
        };
    },
    computed: {
        emailValid() {
            return this.is_email
                ? validateEmail(this.email)
                : validateUsername(this.email);
        },
    },
    methods: {
        async reset_password() {
            const { $manRead } = useNuxtApp();
            try {
                await $manRead("/api/v1/auth/reset-password", {
                    method: "POST",
                    body: { email: this.is_email, ident: this.email },
                });
                this.done = true;
            } catch (e) {
                console.error(e);
            }
        },
    },
};
</script>

<template>
    <FloatingContainer>
        <SigninSignInWelcomeMessage message="Reset your password" />
        <div v-if="!done" class="space-y-6">
            <InputFieldBase
                v-if="is_email"
                id="email"
                v-model="email"
                type="email"
                label="Email address"
                auto-complete="email"
                :disabled="false"
                @click:label="() => (is_email = !is_email)"
            />
            <InputFieldBase
                v-else
                id="name"
                v-model="email"
                type="text"
                label="Name"
                auto-complete="email"
                :disabled="false"
                @click:label="() => (is_email = !is_email)"
            />
            <ForgotPassword name="Remember password?" to="/sign-in" />
            <PrimaryButton
                name="Reset Password"
                :enabled="emailValid"
                @click="reset_password"
            />
        </div>
        <Warn v-else message="Verification code was sent" color="green">
            <div class="text-green-700 text-green-800 bg-green-50" />
        </Warn>
    </FloatingContainer>
</template>

<style scoped></style>
