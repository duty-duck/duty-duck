import type { UseFetchOptions } from "#app";
import type { CommentIncidentRequest } from "bindings/CommentIncidentRequest";
import type { CommentPayload } from "bindings/CommentPayload";
import type { GetIncidentResponse } from "bindings/GetIncidentResponse";
import type { GetIncidentTimelineParams } from "bindings/GetIncidentTimelineParams";
import type { GetIncidentTimelineResponse } from "bindings/GetIncidentTimelineResponse";
import type { ListIncidentsParams } from "bindings/ListIncidentsParams";
import type { ListIncidentsResponse } from "bindings/ListIncidentsResponse";
import type { FetchOptions } from "ofetch";

export const useIncidentRepository = async () => {
    const $fetch = await useServer$fetch();

    return {
        async useIncidents(params: Ref<ListIncidentsParams> | ListIncidentsParams, opts?: UseFetchOptions<ListIncidentsResponse>) {
            return await useServerFetch<ListIncidentsResponse>(`/incidents`, { retry: 3, retryDelay: 5000, query: params, dedupe: "cancel", ...(opts || {}) });
        },
        async useOngoingIncidentsCount() {
            let res = await this.useIncidents({
                status: [
                    "ongoing"
                ],
                priority: null,
                pageNumber: 1,
                itemsPerPage: 1,
                fromDate: null,
                toDate: null,
                orderBy: null,
                orderDirection: null
            }, { lazy: true });
            return { refresh: res.refresh, data: computed(() => res.data.value?.totalNumberOfFilteredResults) }
        },
        async useIncident(incidentId: string, opts?: UseFetchOptions<GetIncidentResponse>) {
            return await useServerFetch<GetIncidentResponse>(`/incidents/${incidentId}`, { retry: 3, retryDelay: 5000, ...(opts || {}) });
        },
        async getIncidentTimeline(incidentId: string, params: Ref<GetIncidentTimelineParams> | GetIncidentTimelineParams) {
            return await $fetch<GetIncidentTimelineResponse>(`/incidents/${incidentId}/events`, { query: params });
        },
        async acknowledgeIncident(incidentId: string) {
            return await $fetch<void>(`/incidents/${incidentId}/acknowledge`, { method: "POST" });
        },
        async commentIncident(incidentId: string, request: CommentIncidentRequest) {
            return await $fetch<void>(`/incidents/${incidentId}/comment`, { method: "POST", body: request });
        }
    }
}