import { createSharedComposable } from "@vueuse/core";
import type { GetProfileResponse } from "bindings/GetProfileResponse"
import type { Permission } from "bindings/Permission"
import type { UnwrapRef } from "vue"

/**
 * Composable for handling authentication and user profile management.
 * @returns An object containing authentication state and methods.
 */
export const useAuth = createSharedComposable(() => {
    const app = useNuxtApp();
    if (!app.$keycloak) {
        throw new Error(`Keycloak is not initialized. Rendered on the server page: ${!import.meta.client}. This composable cannot be used on the server. Check your nuxt.config.ts file.`)
    }
    const { keycloakState, keycloakIsReady, updateToken, logout, login } = app.$keycloak;
    const userRepo = useUserRepository();

    const can = (permission: Permission | Permission[]) => {
        const user = userRepo.userProfile.value;
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
        keycloakIsReady,
        isLoading: computed(() => !userRepo.userProfile.value || userRepo.userProfile.value == "loading"),
        userProfile: computed(() => {
            if (keycloakState.value) {
                return userRepo.userProfile.value
            } else {
                return null
            }
        }),
        userName: computed(() => {
            const user = userRepo.userProfile.value;
            if (user && user != "loading") {
                return `${user.user.firstName} ${user.user.lastName}`
            } else {
                return ""
            }
        }),

        logout,
        login,
        updateToken,
        /**
         * Checks if the user has a specific permission.
         * @param permission - The permission to check.
         * @returns A computed boolean indicating if the user has the permission.
         */
        can,
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
            return navigateTo(localePath("/dashboard"))
        }
    })
}

/** A composable that exposes the current authenticated user,
but also triggers the login sequence if the user is not authenticated
*/
export const useAuthMandatory = async () => {
    const userRepo = useUserRepository();
    const auth = useAuth();
    const { locale } = useI18n();

    await auth.keycloakIsReady;
    if (userRepo.userProfile.value === null) {
        console.log("useAuthMandatory: user is not authenticated, logging in");
        if (!auth.keycloakState) {
            await auth.login({ locale: locale.value })
        }
        console.log("useAuthMandatory: refreshing user profile");
        await userRepo.refreshUserProfile();
    }

    // here's a little trick to make Typescript aware that at this point in runtime, the user profile cannot be null
    return auth as typeof auth & { userProfile: UnwrapRef<GetProfileResponse> }
}
