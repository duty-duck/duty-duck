import { createSharedComposable } from "@vueuse/core"
import type { GetProfileResponse } from "bindings/GetProfileResponse"
import type { Permission } from "bindings/Permission"
import type { User } from "bindings/User"
import Keycloak, { type KeycloakLoginOptions, type KeycloakTokenParsed } from "keycloak-js"
import type { UnwrapRef } from "vue"

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

/**
 * Composable for handling authentication and user profile management.
 * @returns An object containing authentication state and methods.
 */
export const useAuth = createSharedComposable(() => {
    const runtimeConfig = useRuntimeConfig();
    const keycloakIsReady = ref(false);
    const userRepo = useUserRepository();
    const userProfile = ref<GetProfileResponse | "loading" | null>(null);
    const refreshUserProfile = async () => {
        userProfile.value = "loading";
        userProfile.value = await userRepo.fetchUserProfile();
    }
    const keycloakState = ref<KeycloakState | null>(null);
    let keycloak: Keycloak | null;

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

    if (import.meta.client) {
        // Client-side Keycloak initialization
        keycloak =
            new Keycloak({
                url: runtimeConfig.public.keycloakUrl,
                realm: runtimeConfig.public.keycloakRealm,
                clientId: runtimeConfig.public.keycloakClient,
            });

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

            refreshUserProfile();
        };
        keycloak.onAuthLogout = () => {
            keycloakState.value = null;
        }
        keycloak.onTokenExpired = () => {
            updateToken();
        }
        keycloak.init({
            responseMode: 'query',
            flow: 'standard'
        })

    }

    const can = (permission: Permission | Permission[]) => {
        const user = userProfile.value;
        let output = true;
        const permissions: Permission[] = typeof permission == "string" ? [permission] : permission;
        for (let index = 0; index < permissions.length; index++) {
            const hasPermission = user && user !== "loading" && user.permissions.includes(permissions[index]);
            if (!hasPermission) {
                output = false;
            }
        }
        return output;
    };

    return reactive({
        keycloakState,
        /**
         * Logs out the current user.
         */
        async logout(redirectUri?: string) {
            await keycloak!.logout({

                redirectUri: redirectUri ?? `${location.protocol}//${location.host}`
            })
        },
        /**
         * Initiates the login process.
         * @param options - Optional Keycloak login options.
         */
        async login(options?: KeycloakLoginOptions) {
            await keycloak!.login(options)
        },
        keycloakIsReady,
        isLoading: computed(() => !keycloakIsReady.value || userProfile.value == "loading"),
        userProfile: computed(() => {
            if (keycloakState.value) {
                return userProfile.value
            } else {
                return null
            }
        }),
        userName: computed(() => {
            const user = userProfile.value;
            if (user && user != "loading") {
                return `${user.user.firstName} ${user.user.lastName}`
            } else {
                return ""
            }
        }),
        refreshUserProfile: async () => {
            if (keycloakState.value != null) {
                await refreshUserProfile()
            }
        },
        can,
        /**
         * Checks if the user has a specific permission.
         * @param permission - The permission to check.
         * @returns A computed boolean indicating if the user has the permission.
         */
        canComputed(permission: Permission | Permission[]) {
            return computed(() => can(permission))
        }
    });
});

/**
 * Ensures the user has the required permission(s) before mounting the component.
 * Redirects to the dashboard if the user lacks the necessary permissions.
 * @param permission - A single permission or an array of permissions to check.
 */
export const ensurePemissionOnBeforeMount = (permission: Permission | Permission[]) => {
    const { can } = useAuth();
    const router = useRouter();
    const localePath = useLocalePath();
    const permissions: Permission[] = typeof permission == "string" ? [permission] : permission;
    const { show } = useToast();
    const { t } = useI18n();

    onBeforeMount(() => {
        console.log("checking permissions", permissions);

        if (!can(permissions)) {
            show?.({
                props: {
                    title: t('permissions.deniedToastNotification.title'),
                    body: t('permissions.deniedToastNotification.body'),
                    variant: 'danger',
                    value: 5000
                }
            })
            return router.replace(localePath("/dashboard"))
        }
    })
}

/** A composable that exposes the current authenticated user,
but also triggers the login sequence if the user is not authenticated
*/
export const useAuthMandatory = () => {
    const auth = useAuth();
    const { locale } = useI18n();

    watch(
        () => auth.isLoading,
        (isLoading) => {
            if (!isLoading && !auth.userProfile) {
                auth.login({ locale: locale.value })
            }
        },
        { immediate: true }
    );

    // here's a little trick to make Typescript aware that at this point in runtime, the user profile cannot be null
    return auth as typeof auth & { userProfile: UnwrapRef<GetProfileResponse> }
}
