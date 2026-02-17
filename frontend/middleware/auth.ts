import { getAccessToken } from "~/utils/auth";

export default defineNuxtRouteMiddleware(async () => {
  const token = await getAccessToken();
  if (!token) {
    navigateTo("/sign-in");
  }
  // Store tokens in cookies (HTTP-Only cookies can only be set in API routes)
  const accessCookie = useCookie("access_token");
  const refreshCookie = useCookie("refresh_token");

  // Store tokens in state for use in other parts of the app
});
