import type { User } from 'bindings/User'
import Keycloak, { type KeycloakLoginOptions, type KeycloakTokenParsed } from 'keycloak-js'
import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'

export type AuthState =
  | {
    idToken: {
      raw: string,
      parsed: KeycloakTokenParsed & User
    }
    accessToken: {
      raw: string,
      parsed: KeycloakTokenParsed
    }
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

  const updateUser = (user: User) => {
    state.value = {
      status: 'authenticated',
      idToken: {
        raw: keycloak!.idToken!,
        // @ts-ignore
        parsed: {
          ...keycloak!.idTokenParsed!,
          firstName: user.firstName!,
          lastName: user.lastName!,
          email: user.email!,
          phoneNumber: user.phoneNumber,
        }
      },
      accessToken: {
        raw: keycloak!.token!,
        parsed: keycloak!.tokenParsed!
      },
    }
  }

  const updateToken = async () => {
    try {
      await keycloak!.updateToken(30);
      console.log('Auth token refreshed')
      state.value = {
        status: 'authenticated',
        idToken: {
          raw: keycloak!.idToken!,
          // @ts-ignore
          parsed: keycloak!.idTokenParsed!
        },
        accessToken: {
          raw: keycloak!.token!,
          parsed: keycloak!.tokenParsed!
        },
      }
    } catch (e) {
      state.value = { status: 'sessionExpired' }

    }
  }

  if (import.meta.client) {
    // Instantiate the Keycloak client
    keycloak = new Keycloak({
      url: runtimeConfig.public.keycloakUrl,
      realm: runtimeConfig.public.keycloakRealm,
      clientId: runtimeConfig.public.keycloakClient,
    })

    // Modify the login URL to include the `prompt=select_account` parameter that triggers the Active organization authenticator
    // See https://github.com/p2-inc/keycloak-orgs/blob/main/docs/active-organization-authenticator.md for documentation.
    const originalKeycloakCreateLoginUrl = keycloak.createLoginUrl
    keycloak.createLoginUrl = (options: KeycloakLoginOptions) => {
      return `${originalKeycloakCreateLoginUrl(options)}&prompt=select_account`
    }

    keycloak.onReady = () => {
      isReady.value = true
    }
    keycloak.onAuthSuccess = () => {
      state.value = {
        status: 'authenticated',
        idToken: {
          raw: keycloak!.idToken!,
          // @ts-ignore
          parsed: keycloak!.idTokenParsed!
        },
        accessToken: {
          raw: keycloak!.token!,
          parsed: keycloak!.tokenParsed!
        },
      }
    }
    keycloak.onAuthLogout = () => {
      state.value = null
    }
    keycloak.onTokenExpired = () => {
      updateToken();
    }
    keycloak.init({
      responseMode: 'query',
      flow: 'standard'
    })
  }


  const login = async (options?: KeycloakLoginOptions) => {
    await keycloak!.login(options)
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
    onReady,
    updateToken,
    updateUser
  }
})

/** A composable that exposes the current authenticated user,
but also triggers the login sequence if the user is not authenticated
*/
export const useAuthMandatory = () => {
  const authStore = useAuth()
  const { locale } = useI18n();
  authStore.onReady(() => {
    if (!authStore.isAuthenticated) {
      authStore.login({ locale: locale.value })
    }
  })

  return authStore as typeof authStore & { state: AuthState & { status: 'authenticated' } };
}
