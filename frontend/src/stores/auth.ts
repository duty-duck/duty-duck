import Keycloak, { type KeycloakTokenParsed } from 'keycloak-js'
import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'

// Instantiate the Keycloak client
const keycloak = new Keycloak({
  url: import.meta.env.VITE_KEYCLOAK_URL,
  realm: import.meta.env.VITE_KEYCLOAK_REALM,
  clientId: import.meta.env.VITE_KEYCLOAK_CLIENT
})

// Modify the login URL to include the `prompt=select_account` parameter that triggers the Active organization authenticator
// See https://github.com/p2-inc/keycloak-orgs/blob/main/docs/active-organization-authenticator.md for documentation.
const originalKeycloakCreateLoginUrl = keycloak.createLoginUrl
keycloak.createLoginUrl = (options) => {
  return `${originalKeycloakCreateLoginUrl(options)}&prompt=select_account`
}

export type AuthState =
  | {
      idToken: KeycloakTokenParsed
      accessToken: KeycloakTokenParsed
      status: 'authenticated'
    }
  | {
      status: 'sessionExpired'
    }
  | null

/**
 *  A composable that exposes the current authenticated user
 */
export const useAuth = defineStore('auth', () => {
  const isReady = ref(false)
  const state = ref<AuthState>(null)
  const isAuthenticated = computed(() => state.value && state.value.status == 'authenticated')

  const login = async () => {
    await keycloak.login()
  }
  const logout = async () => {
    await keycloak.logout({
      redirectUri: `${location.protocol}//${location.host}`
    })
  }
  const onReady = (callback: () => void) => {
    watch(
      isReady,
      (isReady) => {
        if (isReady) {
          callback()
        }
      },
      { immediate: true }
    )
  }
  keycloak.onReady = () => {
    isReady.value = true
  }
  keycloak.onAuthSuccess = () => {
    state.value = {
      status: 'authenticated',
      idToken: keycloak.idTokenParsed!,
      accessToken: keycloak.tokenParsed!
    }
  }
  keycloak.onAuthLogout = () => {
    state.value = null
  }
  keycloak.onTokenExpired = () => {
    keycloak
      .updateToken(30)
      .then(() => {
        console.log('Auth token refreshed')
        state.value = {
          status: 'authenticated',
          idToken: keycloak.idTokenParsed!,
          accessToken: keycloak.tokenParsed!
        }
      })
      .catch(() => {
        state.value = { status: 'sessionExpired' }
      })
  }
  keycloak.init({
    responseMode: 'query',
    flow: 'standard'
  })

  return {
    isReady,
    state,
    isAuthenticated,
    login,
    logout,
    onReady
  }
})

/** A composable that exposes the current authenticated user,
but also triggers the login sequence if the user is not authenticated
*/
export const useAuthMandatory = () => {
  const authStore = useAuth()
  authStore.onReady(() => {
    if (!authStore.isAuthenticated) {
      authStore.login()
    }
  })

  return authStore
}
