import { type ListHttpMonitorsParams } from "bindings/ListHttpMonitorsParams.js"
import type { ListHttpMonitorsResponse } from "bindings/ListHttpMonitorsResponse";
import type { CreateHttpMonitorCommand } from "bindings/CreateHttpMonitorCommand";
import type { CreateHttpMonitorResponse } from "bindings/CreateHttpMonitorResponse";

export const useHttpMonitorRepository = () => {
    const $fetch = useServer$fetch();
    return {
        async useHttpMonitors(params: Ref<ListHttpMonitorsParams> | ListHttpMonitorsParams) {
            return await useServerFetch<ListHttpMonitorsResponse>(`/http-monitors`, { retry: 3, retryDelay: 5000, query: params });
        },
        async createHttpMonitor(command: CreateHttpMonitorCommand) {
            return await $fetch<CreateHttpMonitorResponse>('/http-monitors', { method: "post", body: command })
        },
        async useDownMonitorsCount() {
            let res = await this.useHttpMonitors({
                include: [
                    "down"
                ], 
                pageNumber: 1,
                itemsPerPage: 0,
                query: null
            });
            return { refresh: res.refresh, data: computed(() => res.data.value?.totalNumberOfFilteredResults) }
        }
    }
}