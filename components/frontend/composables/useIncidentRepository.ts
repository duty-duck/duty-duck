import type { UseFetchOptions } from "#app";
import type { CommentIncidentRequest } from "bindings/CommentIncidentRequest";
import type { CommentPayload } from "bindings/CommentPayload";
import type { FilterableMetadata } from "bindings/FilterableMetadata";
import type { GetIncidentResponse } from "bindings/GetIncidentResponse";
import type { GetIncidentTimelineParams } from "bindings/GetIncidentTimelineParams";
import type { GetIncidentTimelineResponse } from "bindings/GetIncidentTimelineResponse";
import type { ListIncidentsParams } from "bindings/ListIncidentsParams";
import type { ListIncidentsResponse } from "bindings/ListIncidentsResponse";
import type { FetchOptions } from "ofetch";
import { N } from "vitest/dist/chunks/reporters.WnPwkmgA.js";

export const useIncidentRepository = () => {
    return {
        async useFilterableMetadataFields() {
            return await useServerFetch<FilterableMetadata>("/incidents/filterable-metadata");
        },
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
                orderDirection: null,
                metadataFilter: null
            }, { lazy: true });
            return { refresh: res.refresh, data: computed(() => res.data.value?.totalNumberOfFilteredResults) }
        },
        async useIncident(incidentId: string, opts?: UseFetchOptions<GetIncidentResponse>) {
            const $fetch = await useServer$fetch();
            return await useServerFetch<GetIncidentResponse>(`/incidents/${incidentId}`, { retry: 3, retryDelay: 5000, ...(opts || {}) });
        },
        async getIncidentTimeline(incidentId: string, params: Ref<GetIncidentTimelineParams> | GetIncidentTimelineParams) {
            const $fetch = await useServer$fetch();
            return await $fetch<GetIncidentTimelineResponse>(`/incidents/${incidentId}/events`, { query: params });
        },
        async acknowledgeIncident(incidentId: string) {
            const $fetch = await useServer$fetch();
            return await $fetch<void>(`/incidents/${incidentId}/acknowledge`, { method: "POST" });
        },
        async commentIncident(incidentId: string, request: CommentIncidentRequest) {
            const $fetch = await useServer$fetch();
            return await $fetch<void>(`/incidents/${incidentId}/comment`, { method: "POST", body: request });
        }
    }
}