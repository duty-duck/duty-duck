/** A composable giving access to a custom fetch instance
 * The custom fetch instance sets the base url for all API calls and automatically sends the Authorization error
 */
export const useServer$fetch = async () => {
    const { public: { serverUrl } } = useRuntimeConfig();
    const keycloak = await useKeycloak()

    return $fetch.create({
        baseURL: serverUrl,
        onRequest: async ({ options }) => {
            const keycloakState = keycloak.keycloakState.value;
            options.headers = new Headers(options.headers);

            if (keycloakState && keycloakState.accessToken.raw) {
                options.headers.set("Authorization", `Bearer ${keycloakState.accessToken.raw}`);
            } else {
                console.error("No access token found in keycloak state. Cannot fetch protected data");
                await keycloak.login();
            }
        }
    })
}

/**
 * A composable used to fetch data from the Rust API. Compatible with the `useFetch` composable from Vue,
 * this composable sets the base URL for all calls to the URL of the server, as provided by the runtime config,
 * and autoamtically sends the authorization header for all requests.
 */
// @ts-ignore
export const useServerFetch: typeof useFetch = async (request, opts?) => {
    const fetch = await useServer$fetch();
    const keycloak = await useKeycloak();

    return useFetch(request, { $fetch: fetch, ...opts }).then(async result => {
        if (result.error.value) {
            if (result.error.value.statusCode === 401) {
                await keycloak.login();
            }

            console.error("Fetch failed. Request:", request, "Error:", result.error.value);
            throw createError({
                statusMessage: "Fetch failed",
                statusCode: result.error.value.statusCode,
            });
        }
        return result
    })
}