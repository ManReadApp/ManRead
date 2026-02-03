<script lang="ts">

import ForgotPassword from "~/components/signin/ForgotPassword.vue";
import Warn from "~/components/hints/Warn.vue";
import OtpInput from "vue3-otp-input";
import {setTokens} from "~/utils/auth";
import {navigateTo} from "#imports";
import FloatingContainer from "~/components/container/FloatingContainer.vue";
import PrimaryButton from "~/components/button/PrimaryButton.vue";
import InputFieldBase from "~/components/input/InputFieldBase.vue"

export default {
  components: {
    OtpInput,
    Warn,
    ForgotPassword,
    InputFieldBase, FloatingContainer, PrimaryButton
  },
  setup() {
    useHead({
      title: "Reset Password - ManRead",
    });
    const route = useRoute();
    const ident = route.query.ident;
    const token = route.query.token;
    return {
      ident: ref(ident),
      key: ref(token),
      password: ref(""),
      loading: ref(false)
    };
  },
  computed: {
    should_warn() {
      return !(this.key && !Array.isArray(this.key) && this.key.length == 6 && this.ident && !Array.isArray(this.ident) && validateEmail(this.ident as string))
    },
    validPassword() {
      return !this.loading && validatePassword(this.password);
    }
  },
  methods: {
    async reset_password() {
      const {$manRead} = useNuxtApp()
      try {
        const resp= await $manRead('/api/v1/auth/verify-reset-password', {
          method: "PUT",
          body: {
            email: true,
            ident: Array.isArray(this.ident) ? '' : this.ident ?? '',
            key: Array.isArray(this.key) ? '' : this.key ?? '',
            password: this.password
          }
        });

        await setTokens(resp.access_token, resp.refresh_token)
        navigateTo("/");
      } catch (e) {
        console.error(e);
      }
    }
  }
}
</script>

<template>
  <FloatingContainer>
    <SigninSignInWelcomeMessage message="Reset your password"/>
    <Warn v-if="should_warn" message="No email | token provided" color="yellow">
      <div class="yellow-green-700 text-yellow-800 bg-yellow-50" />
    </Warn>
    <div v-else class="space-y-6">
      <InputFieldBase id="name" type="text" label="Password"  v-model="password"
                  auto-complete="off" :disabled="loading"/>
      <PrimaryButton name="Reset Password" :enabled="validPassword" @click="reset_password"/>
    </div>
  </FloatingContainer>
</template>

<style scoped>

</style>