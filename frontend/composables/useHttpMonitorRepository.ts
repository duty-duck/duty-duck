import { type PaginationParams } from "bindings/PaginationParams"
import type { ListHttpMonitorsResponse } from "bindings/ListHttpMonitorsResponse";
import type { CreateHttpMonitorCommand } from "bindings/CreateHttpMonitorCommand";
import type { CreateHttpMonitorResponse } from "bindings/CreateHttpMonitorResponse";

export const useHttpMonitorRepository = () => {
    const $fetch = useServer$fetch();
    return {
        async useHttpMonitors(params: Ref<PaginationParams>) {
            return await useServerFetch<ListHttpMonitorsResponse>(`/http-monitors`, { retry: 3, retryDelay: 5000, query: params });
        },
        async createHttpMonitor(command: CreateHttpMonitorCommand) {
            return await $fetch<CreateHttpMonitorResponse>('/http-monitors', { method: "post", body: command })
        }
    }
}