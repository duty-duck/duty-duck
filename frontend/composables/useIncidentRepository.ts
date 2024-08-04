import type { ListIncidentsParams } from "bindings/ListIncidentsParams";
import type { ListIncidentsResponse } from "bindings/ListIncidentsResponse";

export const useIncidentRepository = () => {
    const $fetch = useServer$fetch();

    return {
        async useIncidents(params: Ref<ListIncidentsParams> | ListIncidentsParams) {
            return await useServerFetch<ListIncidentsResponse>(`/incidents`, { retry: 3, retryDelay: 5000, query: params, dedupe: "defer" });
        },
        async useOngoingIncidentsCount() {
            let res = await this.useIncidents({
                status: [
                    "ongoing"
                ], priority: null, pageNumber: 1, itemsPerPage: 0
            });
            return { refresh: res.refresh, data: computed(() => res.data.value?.totalNumberOfFilteredResults) }
        }
    }
}