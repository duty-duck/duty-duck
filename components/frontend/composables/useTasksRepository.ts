import type { UseFetchOptions } from "#app"
import type { ListTaskRunsParams } from "bindings/ListTaskRunsParams"
import type { ListTaskRunsResponse } from "bindings/ListTaskRunsResponse"
import type { ListTasksParams } from "bindings/ListTasksParams"
import type { ListTasksResponse } from "bindings/ListTasksResponse"

export const useTasksRepository = () => {
    return {
        async useTasks(params: ListTasksParams | Ref<ListTasksParams>, opts?: UseFetchOptions<ListTasksResponse>) {
            return useServerFetch<ListTasksResponse>("/tasks", { query: params, retry: 3, dedupe: "cancel", ...(opts || {}) })
        },
        async useTaskRuns(taskId: string, params?: ListTaskRunsParams | Ref<ListTaskRunsParams>, opts?: UseFetchOptions<ListTaskRunsResponse>) {
            return useServerFetch<ListTaskRunsResponse>(`/tasks/${taskId}/runs`, { query: params, retry: 3, dedupe: "cancel", ...(opts || {}) })
        }
    }
}