import { type ListHttpMonitorsParams } from "bindings/ListHttpMonitorsParams.js"
import type { ListHttpMonitorsResponse } from "bindings/ListHttpMonitorsResponse";
import type { CreateHttpMonitorCommand } from "bindings/CreateHttpMonitorCommand";
import type { CreateHttpMonitorResponse } from "bindings/CreateHttpMonitorResponse";
import type { ReadHttpMonitorResponse } from "bindings/ReadHttpMonitorResponse";
import type { ListIncidentsResponse } from "bindings/ListIncidentsResponse";
import type { ListIncidentsParams } from "bindings/ListIncidentsParams";
import type { UpdateHttpMonitorCommand } from "bindings/UpdateHttpMonitorCommand";
import type { UseFetchOptions } from "#app";
import type { FilterableMetadata } from "bindings/FilterableMetadata";

export const useHttpMonitorRepository = () => {
    return {
        async useFilterableMetadataFields() {
            return await useServerFetch<FilterableMetadata>("/http-monitors/filterable-metadata");
        },
        async useHttpMonitors(params: Ref<ListHttpMonitorsParams> | ListHttpMonitorsParams) {
            return await useServerFetch<ListHttpMonitorsResponse>(`/http-monitors`, { retry: 3, retryDelay: 5000, query: params });
        },
        async createHttpMonitor(command: CreateHttpMonitorCommand) {
            const $fetch = await useServer$fetch();
            return await $fetch<CreateHttpMonitorResponse>('/http-monitors', { method: "post", body: command })
        },
        async updateHttpMonitor(monitorId: string, command: UpdateHttpMonitorCommand) {
            const $fetch = await useServer$fetch();
            return await $fetch<void>(`/http-monitors/${monitorId}`, { method: "patch", body: command })
        },
        async toggleHttpMonitor(monitorId: string) {
            const $fetch = await useServer$fetch();
            return await $fetch<void>(`/http-monitors/${monitorId}/toggle`, { method: "post" })
        },
        async useHttpMonitor(monitorId: string, options?: UseFetchOptions<ReadHttpMonitorResponse>) {
            return await useServerFetch<ReadHttpMonitorResponse>(`/http-monitors/${monitorId}`, { retry: 3, retryDelay: 5000, ...(options || {}) })
        },
        async useHttpMonitorIncidents(monitorId: string, params: Ref<ListIncidentsParams> | ListIncidentsParams, options?: UseFetchOptions<ListIncidentsResponse>) {
            return await useServerFetch<ListIncidentsResponse>(`/http-monitors/${monitorId}/incidents`, { retry: 3, retryDelay: 5000, params, ...(options || {}) })
        },
        async useDownMonitorsCount() {
            let res = await this.useHttpMonitors({
                include: [
                    "down"
                ],
                pageNumber: 1,
                itemsPerPage: 0,
                query: null,
                metadataFilter: null
            });
            return { refresh: res.refresh, data: computed(() => res.data.value?.totalNumberOfFilteredResults) }
        }
    }
}