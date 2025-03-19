import type { Permission } from "bindings/Permission";

export default defineNuxtRouteMiddleware(async (to, from) => {
  const routePermissions = to.meta.permissions as Permission[] | undefined;
  if (routePermissions) {
    const auth = await useAuth();

    if (!auth.userHasPermission(routePermissions)) {
      console.log("User is lacking permission", routePermissions, "to navigate to", to.fullPath);
      throw abortNavigation(createError({ statusCode: 403, message: "Forbidden", fatal: true }));
    }
  }
});