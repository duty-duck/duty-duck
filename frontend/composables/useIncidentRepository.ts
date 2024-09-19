import type { UseFetchOptions } from "#app";
import type { ListIncidentsParams } from "bindings/ListIncidentsParams";
import type { ListIncidentsResponse } from "bindings/ListIncidentsResponse";

export const useIncidentRepository = () => {
    const $fetch = useServer$fetch();

    return {
        async useIncidents(params: Ref<ListIncidentsParams> | ListIncidentsParams, opts?: UseFetchOptions<ListIncidentsResponse>) {
            return await useServerFetch<ListIncidentsResponse>(`/incidents`, { retry: 3, retryDelay: 5000, query: params, dedupe: "defer", ...(opts || {}) });
        },
        async useOngoingIncidentsCount() {
            let res = await this.useIncidents({
                status: [
                    "ongoing"
                ],
                priority: null,
                pageNumber: 1,
                itemsPerPage: 1
            }, { lazy: true });
            return { refresh: res.refresh, data: computed(() => res.data.value?.totalNumberOfFilteredResults) }
        }
    }
}