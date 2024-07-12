/** A composable giving access to a custom fetch instance
 * The custom fetch instance sets the base url for all API calls and automatically sends the Authorization error
 */
export const useServer$fetch = () => {
    const { public: { serverUrl } } = useRuntimeConfig();

    return $fetch.create({
        baseURL: serverUrl,
        onRequest: ({ options }) => {
            const { state: auth } = useAuth();
            options.headers = options.headers || {};
            if (auth?.status == "authenticated") {
                // @ts-ignore
                options.headers["Authorization"] = `Bearer ${auth.accessToken.raw}`;
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

    return useFetch(request, { $fetch: fetch }).then(result => {
        if (result.error.value) {
            console.error("Fetch error: ", result.error.value)
        }
        return result
    })
}