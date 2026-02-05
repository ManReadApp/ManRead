import { useCookie, useState } from "#app";

export async function setTokens(accessToken: string, refreshToken: string) {
  const accessToken_ = useState("access_token");
  const refreshToken_ = useCookie("refresh_token", {
    path: "/",
    sameSite: "lax",
  });
  accessToken_.value = accessToken;
  refreshToken_.value = refreshToken;
}

export function generateClaim(access_key: string): null | Claim {
  const parts = access_key.split(".");
  if (parts.length !== 3) {
    return null;
  }
  return JSON.parse(atob(parts[1]!));
}

export function sessionValid() {
  const refreshToken = useCookie("refresh_token", {
    path: "/",
    sameSite: "lax",
  });
  if (!refreshToken.value) {
    return false;
  }
  const claim = generateClaim(refreshToken.value);
  if (!claim) {
    return false;
  }
  return claim.exp > Date.now();
}

export async function getAccessToken(): Promise<null | string> {
  const accessToken = useState<string | null>("access_token");
  const refreshToken = useCookie("refresh_token", {
    path: "/",
    sameSite: "lax",
  });

  if (accessToken.value) {
    const claim = generateClaim(accessToken.value)!;
    if (claim.exp > Date.now()) {
      return accessToken.value;
    } else {
      accessToken.value = null;
    }
  }

  if (refreshToken.value) {
    const { $manRead } = useNuxtApp();
    try {
      const data = await $manRead("/api/v1/auth/refresh", {
        method: "POST",
        body: { refresh_token: refreshToken.value },
      });
      accessToken.value = data.access_token;
      refreshToken.value = data.refresh_token;
      refreshCookie("refresh_token");

      return accessToken.value;
    } catch (e) {
      console.error("Failed to refresh token", e.toString());
      console.error("message:", e?.message);
      return null;
    }
  }

  return null;
}
