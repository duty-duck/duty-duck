/** A composable giving access to a custom fetch instance
 * The custom fetch instance sets the base url for all API calls and automatically sends the Authorization error
 */
export const useServer$fetch = () => {
    const { public: { serverUrl } } = useRuntimeConfig();

    return $fetch.create({
        baseURL: serverUrl,
        onRequest: ({ options }) => {
            const auth = useAuth();
            options.headers = options.headers || {};
            if (!auth?.keycloakState != null) {
                // @ts-ignore
                options.headers["Authorization"] = `Bearer ${auth.keycloakState.accessToken.raw}`;
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
export const useServerFetch: typeof useFetch = (request, opts?) => {
    const fetch = useServer$fetch();

    return useFetch(request, { $fetch: fetch, ...opts }).then(result => {
        if (result.error.value) {
            throw createError({
                statusMessage: "Fetch failed",
                statusCode: result.error.value.statusCode,
            });
        }
        return result
    })
}

// @ts-ignore
export const useLazyServerFetch: typeof useLazyFetch = (request, opts?) => {
    const fetch = useServer$fetch();

    return useLazyFetch(request, { $fetch: fetch, ...opts }).then(result => {
        if (result.error.value) {
            throw createError({
                statusMessage: "Fetch failed",
                statusCode: result.error.value.statusCode,
            });
        }
        return result
    })
}