<template>
    <FloatingContainer :small="false">
        <div v-if="step1">
            <SigninSignInWelcomeMessage message="Sign up for an account" />
            <div class="space-y-6">
                <InputFieldBase
                    v-model="state.username"
                    iid="name"
                    type="text"
                    label="Name"
                    auto-complete="off"
                    :open-button="false"
                    :open-upwards="false"
                    :disabled="loading"
                />
                <InputFieldBase
                    v-model="state.email"
                    iid="email"
                    type="email"
                    label="Email address"
                    :open-button="false"
                    :open-upwards="false"
                    auto-complete="email"
                    :disabled="false"
                />
                <InputFieldBase
                    v-model="state.password"
                    iid="password"
                    type="password"
                    label="Password"
                    :open-button="false"
                    :open-upwards="false"
                    auto-complete="new-password"
                    :disabled="false"
                />
                <InputFieldBase
                    v-model="state.password2"
                    iid="password-confirm"
                    type="password"
                    label="Password Again"
                    :open-button="false"
                    :open-upwards="false"
                    auto-complete="new-password"
                    :disabled="false"
                />
                <ForgotPassword name="Already have an account?" to="/sign-in" />
                <PrimaryButton
                    name="Sign Up"
                    :enabled="next_available"
                    @click="() => (step1 = false)"
                />
            </div>
        </div>
        <div v-else>
            <div class="w-full flex mb-2">
                <button
                    class="bg-gray-200 hover:bg-gray-300 outline-gray-200 hover:outline-gray-300 cursor-pointer text-gray-800 font-bold py-2 px-4 rounded-l-lg w-1/3 flex items-center justify-center outline-2 -outline-offset-2"
                    :class="
                        gender == 'Male'
                            ? 'outline-indigo-600 hover:outline-indigo-600'
                            : ''
                    "
                    @click="() => (loading ? null : (gender = 'Male'))"
                >
                    <NuxtImg src="/icons/sex/male.svg" class="h-24" />
                </button>
                <button
                    class="bg-gray-200 hover:bg-gray-300 outline-gray-200 hover:outline-gray-300 cursor-pointer text-gray-800 font-bold py-2 px-4 flex-1 w-1/3 flex items-center justify-center outline-2 -outline-offset-2"
                    :class="
                        gender == 'Female'
                            ? 'outline-indigo-600 hover:outline-indigo-600'
                            : ''
                    "
                    @click="() => (loading ? null : (gender = 'Female'))"
                >
                    <NuxtImg src="/icons/sex/female.svg" class="h-24" />
                </button>
                <button
                    class="bg-gray-200 hover:bg-gray-300 outline-gray-200 hover:outline-gray-300 cursor-pointer text-gray-800 font-bold py-2 px-4 rounded-r-lg flex-1 w-1/3 flex items-center justify-center outline-2 -outline-offset-2"
                    :class="
                        gender == 'Unknown'
                            ? 'outline-indigo-600 hover:outline-indigo-600'
                            : ''
                    "
                    @click="() => (loading ? null : (gender = 'Unknown'))"
                >
                    <NuxtImg src="/icons/sex/unknown.svg" class="h-24" />
                </button>
            </div>
            <InputFieldBase
                iid="bddate"
                type="date"
                label="Birthday"
                auto-complete="off"
                model-value=""
                :open-button="false"
                :open-upwards="false"
                :disabled="loading"
            />
            <div class="mt-2" />
            <InputFieldBase
                iid="profile"
                type="file"
                label="Profile picture"
                auto-complete="off"
                model-value=""
                :open-button="false"
                :open-upwards="false"
                :disabled="loading"
            />
            <PrimaryButton
                class="mt-6"
                :enabled="submit_available"
                name="Create Account"
                @click="register"
            />
        </div>
    </FloatingContainer>
</template>
<script lang="ts">
import ForgotPassword from "~/components/signin/ForgotPassword.vue";
import { setTokens } from "~/utils/auth";
import { navigateTo } from "#imports";
import { validateUsername } from "~/utils/validation";
import PrimaryButton from "~/components/button/PrimaryButton.vue";
import FloatingContainer from "~/components/container/FloatingContainer.vue";
import InputFieldBase from "~/components/input/InputFieldBase.vue";

export default {
    components: {
        FloatingContainer,
        PrimaryButton,
        InputFieldBase,
        ForgotPassword,
    },
    setup() {
        useHead({
            title: "SignUp - ManRead",
        });
        return {
            state: reactive({
                username: "",
                email: "",
                password: "",
                password2: "",
                date: new Date(),
            }),
            file: ref<null | -1 | string>(null),
            gender: ref<"Male" | "Female" | "Unknown" | "">(""),
            loading: ref(false),
            step1: ref(true),
        };
    },
    computed: {
        next_available() {
            return (
                this.state.password == this.state.password2 &&
                validateEmail(this.state.email) &&
                validateUsername(this.state.username) &&
                validatePassword(this.state.password)
            );
        },
        submit_available() {
            return !this.loading && this.gender != "" && this.file != -1;
        },
    },
    methods: {
        async upload(files: File[]) {
            this.file = -1;
            const formData = new FormData();
            formData.append("file", files[0]);
            const { data, error } = await useFetch("/api/v1/image/upload", {
                method: "POST",
                body: formData,
            });
            if (
                error.value ||
                !data.value ||
                !Array.isArray(data.value) ||
                !Array.isArray(data.value[0]) ||
                data.value[0].length < 2
            ) {
                this.file = null;
            } else {
                this.file = (data.value as Array<Array<string>>)[0][1];
            }
        },
        async register() {
            if (this.loading || this.gender === "") {
                return;
            }
            this.loading = true;
            //TODO: validate ui & check if username available
            const { $manRead } = useNuxtApp();
            try {
                const resp = await $manRead("/api/v1/auth/register", {
                    method: "PUT",
                    body: {
                        email: this.state.email,
                        name: this.state.username,
                        password: this.state.password,
                        gender: this.gender,
                        birthdate: this.state.date.toISOString(),
                        icon_temp_name: null,
                    },
                });
                await setTokens(resp.access_token, resp.refresh_token);
                navigateTo("/verify");
            } catch (e) {
                console.error(e);
                //TODO:
            }

            this.loading = false;
        },
    },
};
</script>
