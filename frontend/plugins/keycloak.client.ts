import type { GetProfileResponse } from 'bindings/GetProfileResponse';
import type { Permission } from 'bindings/Permission';
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
        'app:created'() {
            const router = useRouter();
            router.currentRoute.value.query = {};
            router.replace({ query: {} });
        },
    },
    setup: async function () {
        const runtimeConfig = useRuntimeConfig();
        const initOptions: KeycloakConfig = {
            url: runtimeConfig.public.keycloakUrl,
            realm: runtimeConfig.public.keycloakRealm,
            clientId: runtimeConfig.public.keycloakClient,
        };
        const keycloakIsReady = ref(false);
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
            console.log("login", options);
            await keycloak!.login(options)
        }

        /**
         * Logs out the current user.
         */
        const logout = async (redirectUri?: string) => {
            await keycloak!.logout({
                redirectUri: redirectUri ?? `${location.protocol}//${location.host}`
            })
        }

        const keycloak = new Keycloak(initOptions);
        // Modify the login URL to include the `prompt=select_account` parameter that triggers the Active organization authenticator
        // See https://github.com/p2-inc/keycloak-orgs/blob/main/docs/active-organization-authenticator.md for documentation.
        const originalKeycloakCreateLoginUrl = keycloak.createLoginUrl
        keycloak.createLoginUrl = (options: KeycloakLoginOptions) => {
            return `${originalKeycloakCreateLoginUrl(options)}&prompt=select_account`
        }
        keycloak.onReady = () => {
            keycloakIsReady.value = true
        }
        keycloak.onAuthSuccess = () => {
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
        keycloak.onAuthLogout = () => {
            keycloakState.value = null;
        }
        keycloak.onTokenExpired = () => {
            updateToken();
        }

        await keycloak.init({
            responseMode: 'query',
            flow: 'standard',
            checkLoginIframe: false,
        });

        return {
            provide: {
                keycloak: {
                    keycloak,
                    keycloakIsReady,
                    keycloakState,
                    updateToken,
                    logout,
                    login,
                }
            },
        };
    },

});