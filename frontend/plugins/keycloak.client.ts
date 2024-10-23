import Keycloak, { type KeycloakConfig, type KeycloakLoginOptions, type KeycloakTokenParsed } from 'keycloak-js';

/**
 * Represents the state of the Keycloak authentication.
 */
type KeycloakState = {
    accessToken: {
        raw: string,
        parsed: KeycloakTokenParsed
    },
    idToken: {
        raw: string,
        parsed: KeycloakTokenParsed
    }
}

export default defineNuxtPlugin({
    name: 'KeycloakClient',
    enforce: 'pre',
    hooks: {
        /**
         * Fix router issue, see : https://github.com/keycloak/keycloak/issues/14742
         */
        // 'app:created'() {
        //     const router = useRouter();
        //     setTimeout(() => {
        //         router.currentRoute.value.query = {};
        //         router.replace({ query: {} });
        //     }, 1000)
        // },
    },
    setup: async function () {
        const runtimeConfig = useRuntimeConfig();

        let keycloak: Keycloak | null = null;

        const initOptions: KeycloakConfig = {
            url: runtimeConfig.public.keycloakUrl,
            realm: runtimeConfig.public.keycloakRealm,
            clientId: runtimeConfig.public.keycloakClient,
        };
        const keycloakState = ref<KeycloakState | null>(null);

        /**
         * Refreshes the user's token.
         */
        const updateToken = async () => {
            try {
                await keycloak!.updateToken(30);
                console.log('Auth token refreshed')
                keycloakState.value = {
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
                keycloakState.value = null
            }
        };

        /**
         * Initiates the login process.
         * @param options - Optional Keycloak login options.
         */
        const login = async (options?: KeycloakLoginOptions) => {
            console.log("[KeycloakClient] Calling 'login'", options);
            await keycloak!.login(options)
        }

        /**
         * Logs out the current user.
         */
        const logout = async (redirectUri?: string) => {
            console.log("[KeycloakClient] Calling 'logout'", redirectUri);
            await keycloak!.logout({
                redirectUri: redirectUri ?? `${location.protocol}//${location.host}`
            })
        }

        const getKeycloakInstance = async () => {
            if (keycloak) {
                return keycloak;
            }

            console.log("[KeycloakClient] Initializing Keycloak");
            return await new Promise<Keycloak>((resolve) => {
                keycloak = new Keycloak(initOptions);

                // Expose the keycloak instance and the keycloak state to the global window object to make it easier to debug in production
                // @ts-ignore
                window.KEYCLOAK = keycloak;
                // @ts-ignore
                window.KEYCLOAK_STATE = keycloakState;

                // Modify the login URL to include the `prompt=select_account` parameter that triggers the Active organization authenticator
                // See https://github.com/p2-inc/keycloak-orgs/blob/main/docs/active-organization-authenticator.md for documentation.
                const originalKeycloakCreateLoginUrl = keycloak.createLoginUrl
                keycloak.createLoginUrl = (options: KeycloakLoginOptions) => {
                    return `${originalKeycloakCreateLoginUrl(options)}&prompt=select_account`
                }
                keycloak.onReady = () => {
                    console.log("[KeycloakClient] Keycloak is ready");
                    resolve(keycloak!);
                }
                keycloak.onAuthSuccess = () => {
                    console.log("[KeycloakClient] Keycloak authentication successful");
                    keycloakState.value = {
                        idToken: {
                            raw: keycloak!.idToken!,
                            // @ts-ignore
                            parsed: keycloak!.idTokenParsed!
                        },
                        accessToken: {
                            raw: keycloak!.token!,
                            parsed: keycloak!.tokenParsed!
                        },
                    };
                };
                keycloak.onAuthError = (e) => {
                    console.error("[KeycloakClient] Keycloak authentication error", e);
                }
                keycloak.onAuthLogout = () => {
                    console.log("[KeycloakClient] Keycloak logout");
                    keycloakState.value = null;
                }
                keycloak.onTokenExpired = () => {
                    updateToken();
                }
                keycloak.init({
                    responseMode: 'query',
                    flow: 'standard',
                    checkLoginIframe: false,
                });
            })
        }

        return {
            provide: {
                keycloak: {
                    keycloak,
                    getKeycloakInstance,
                    keycloakState,
                    updateToken,
                    logout,
                    login,
                }
            },
        };
    },

});