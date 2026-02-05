import { sessionValid } from "~/utils/auth";

export default defineNuxtRouteMiddleware(async () => {
  if (!sessionValid()) {
    navigateTo("/sign-in");
  }
});
