import type { Permission } from "bindings/Permission"

export const useKeycloak = async () => {
    const app = useNuxtApp();
    if (!app.$keycloak) {
        throw new Error(`Keycloak is not initialized. Rendered on the server page: ${!import.meta.client}. This composable cannot be used on the server. Check your nuxt.config.ts file.`)
    }
    const keycloak = app.$keycloak;
    const keycloakInstance =await keycloak.getKeycloakInstance();

    return {
        login: keycloak.login,
        logout: keycloak.logout,
        keycloakState: keycloak.keycloakState,
        keycloakInstance,
    }
};

/**
 * Composable for handling authentication and user profile management.
 * @returns An object containing authentication state and methods.
 */
export const useAuth = async () => {
    const keycloak = await useKeycloak();
    if (!keycloak.keycloakInstance.authenticated) {
        await keycloak.login();
    }
    const userRepo = await useUserRepository();
    const { data: userProfile } = await userRepo.useUserProfile();

    const userHasPermission = (permission: Permission | Permission[]): boolean => {
        const user = userProfile.value!;
        let output = true;
        const permissions: Permission[] = typeof permission == "string" ? [permission] : permission;
        for (let index = 0; index < permissions.length; index++) {
            const hasPermission = user && user.permissions.includes(permissions[index]);
            if (!hasPermission) {
                output = false;
            }
        }
        return output;
    };
    return reactive({
        userProfile: computed(() => {
            return userProfile.value!
        }),
        userName: computed(() => {
            const user = userProfile.value!.user;
            if (user) {
                return `${user.firstName} ${user.lastName}`
            } else {
                return ""
            }
        }),
        userHasPermission,
        userHasPermissionComputed: (permission: Permission | Permission[]) => {
            return computed(() => userHasPermission(permission))
        },
        ...keycloak,
    });
}

/**
 * Ensures the user has the required permission(s) before mounting the component.
 * Redirects to the dashboard if the user lacks the necessary permissions.
 * @param permission - A single permission or an array of permissions to check.
 */
export const usePermissionGrant = async (permission: Permission | Permission[]) => {
    const localePath = useLocalePath();
    const permissions: Permission[] = typeof permission == "string" ? [permission] : permission;
    const { show } = useToast();
    const { t } = useI18n();
    const { userHasPermission } = await useAuth();

    console.log("checking permissions", permissions);

    if (!userHasPermission(permissions)) {
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
}