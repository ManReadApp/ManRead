import {getAccessToken} from "~/utils/auth";

export default defineNuxtRouteMiddleware(async () => {
    const token = await getAccessToken();
    if(!token) {
        navigateTo("/sign-in");
    }
    const accessToken = 'new_access_token_123'
    const refreshToken = 'new_refresh_token_456'

    // Store tokens in cookies (HTTP-Only cookies can only be set in API routes)
    const accessCookie = useCookie('access_token')
    const refreshCookie = useCookie('refresh_token')
    accessCookie.value = accessToken
    refreshCookie.value = refreshToken

    // Store tokens in state for use in other parts of the app
    const authState = useState('auth', () => ({ accessToken: '', refreshToken: '' }))
    authState.value = { accessToken, refreshToken }

    console.log('Tokens set:', authState.value)
})