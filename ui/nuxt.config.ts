// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2025-07-15",
  devtools: { enabled: true },
  openFetch: {
    clients: {
      manRead: {
        baseURL: "http://127.0.0.1:8082",
        schema: "http://127.0.0.1:8082/openapi.json",
      },
    },
  },
  routeRules: {
    "/api/**": {
      proxy: "http://127.0.0.1:8082" + "/api/**",
    },
  },
  modules: [
    "@nuxt/eslint",
    "@nuxt/hints",
    "@nuxt/icon",
    "@nuxt/image",
    "@nuxtjs/tailwindcss",
    "nuxt-open-fetch",
  ],
  css: ["~/assets/css/main.css"],
});
