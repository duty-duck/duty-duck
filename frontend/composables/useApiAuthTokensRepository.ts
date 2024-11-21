import type { CreateApiTokenRequest } from "bindings/CreateApiTokenRequest";
import type { CreateApiTokenResponse } from "bindings/CreateApiTokenResponse";
import type { ListApiAccessTokensResponse } from "bindings/ListApiAccessTokensResponse";

export const useApiAuthTokensRepository = () => {

    return {
        async useApiAuthTokens() {
            return await useServerFetch<ListApiAccessTokensResponse>("/api-tokens");
        },
        async createApiAuthToken(request: CreateApiTokenRequest) {
            const $fetch = await useServer$fetch();
            return await $fetch<CreateApiTokenResponse>("/api-tokens", { method: "post", body: request });
        },
        async deleteApiAuthToken(apiToken: string) {
            const $fetch = await useServer$fetch();
            return await $fetch<void>(`/api-tokens/${apiToken}`, { method: "delete" });
        },
    };
};