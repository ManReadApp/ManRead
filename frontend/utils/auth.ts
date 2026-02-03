import {useCookie, useState} from '#app';

export async function setTokens(accessToken: string, refreshToken: string) {
    const accessToken_ = useState('access_token');
    const refreshToken_ = useCookie('refresh_token');
    const claim = useState<Claim | null>('claim');
    accessToken_.value = accessToken;
    refreshToken_.value = refreshToken;
    claim.value = generateClaim(accessToken);
    refreshCookie('refresh_token')
}

export function generateClaim(access_key: string): null | Claim {
    const parts = access_key.split(".");
    if (parts.length !== 3) {
        return null;
    }
    return JSON.parse(atob(parts[1]))
}


export async function getAccessToken(): Promise<null | string> {
    const accessToken = useState<string | null>('access_token');
    const claim = useState<Claim | null>('claim');
    const refreshToken = useCookie('refresh_token');

    if (accessToken.value && claim.value) {
        if (claim.value.exp > Date.now()) {
            return accessToken.value;
        } else {
            accessToken.value = null;
            claim.value = null;
        }
    }else if (accessToken.value) {
        console.error("couldnt parse claim")
    }

    if (refreshToken.value) {
        const {$manRead} = useNuxtApp();
        try {
            const data = await $manRead('/api/v1/auth/refresh', {
                method: 'POST',
                body: {refresh_token: refreshToken.value},
                credentials: 'include',
            });
            accessToken.value = data.access_token;
            claim.value = generateClaim(data.access_token);
            refreshToken.value = data.refresh_token;
            refreshCookie('refresh_token')
            return accessToken.value;
        } catch (e) {
            console.error('Failed to refresh token', e);
            return null;
        }
    }

    return null;
}
