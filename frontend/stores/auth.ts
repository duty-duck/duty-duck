import Keycloak, { type KeycloakTokenParsed } from 'keycloak-js'
import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'



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
  const runtimeConfig = useRuntimeConfig();
  const isReady = ref(false)
  const state = ref<AuthState>(null)
  const isAuthenticated = computed(() => state.value && state.value.status == 'authenticated')

  let keycloak: Keycloak | null = null;
  if (import.meta.client) {
    // Instantiate the Keycloak client
    keycloak = new Keycloak({
      url: runtimeConfig.public.keycloak.url,
      realm: runtimeConfig.public.keycloak.realm,
      clientId: runtimeConfig.public.keycloak.client
    })

    // Modify the login URL to include the `prompt=select_account` parameter that triggers the Active organization authenticator
    // See https://github.com/p2-inc/keycloak-orgs/blob/main/docs/active-organization-authenticator.md for documentation.
    const originalKeycloakCreateLoginUrl = keycloak.createLoginUrl
    keycloak.createLoginUrl = (options) => {
      return `${originalKeycloakCreateLoginUrl(options)}&prompt=select_account`
    }

    keycloak.onReady = () => {
      isReady.value = true
    }
    keycloak.onAuthSuccess = () => {
      state.value = {
        status: 'authenticated',
        idToken: keycloak!.idTokenParsed!,
        accessToken: keycloak!.tokenParsed!
      }
    }
    keycloak.onAuthLogout = () => {
      state.value = null
    }
    keycloak.onTokenExpired = () => {
      keycloak!
        .updateToken(30)
        .then(() => {
          console.log('Auth token refreshed')
          state.value = {
            status: 'authenticated',
            idToken: keycloak!.idTokenParsed!,
            accessToken: keycloak!.tokenParsed!
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
  }


  const login = async () => {
    await keycloak!.login()
  }
  const logout = async () => {
    await keycloak!.logout({
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
