import type { UseFetchOptions } from "#app"
import type { CreateTaskCommand } from "bindings/CreateTaskCommand"
import type { FilterableMetadata } from "bindings/FilterableMetadata"
import type { GetTaskResponse } from "bindings/GetTaskResponse"
import type { GetTaskRunLogsParams } from "bindings/GetTaskRunLogsParams"
import type { GetTaskRunLogsResponse } from "bindings/GetTaskRunLogsResponse"
import type { GetTaskRunResponse } from "bindings/GetTaskRunResponse"
import type { ListTaskRunsParams } from "bindings/ListTaskRunsParams"
import type { ListTaskRunsResponse } from "bindings/ListTaskRunsResponse"
import type { ListTasksParams } from "bindings/ListTasksParams"
import type { ListTasksResponse } from "bindings/ListTasksResponse"
import type { UpdateTaskCommand } from "bindings/UpdateTaskCommand"
import { FetchError } from "ofetch"

export const useTasksRepository = () => {
    return {
        async useTasks(params?: ListTasksParams | Ref<ListTasksParams>, opts?: UseFetchOptions<ListTasksResponse>) {
            return useServerFetch<ListTasksResponse>("/tasks", { query: params, retry: 3, dedupe: "cancel", ...(opts || {}) })
        },
        async useFilterableMetadataFields() {
            return await useServerFetch<FilterableMetadata>("/tasks/filterable-metadata");
        },
        async useTaskRuns(taskId: string, params?: ListTaskRunsParams | Ref<ListTaskRunsParams>, opts?: UseFetchOptions<ListTaskRunsResponse>) {
            return useServerFetch<ListTaskRunsResponse>(`/tasks/${taskId}/runs`, { query: params, retry: 3, dedupe: "cancel", ...(opts || {}) })
        },
        async useTask(taskId: string) {
            return useServerFetch<GetTaskResponse>(`/tasks/${taskId}`, { retry: 3, dedupe: "cancel" })
        },
        async useTaskRun(taskId: string, runId: string) {
            return useServerFetch<GetTaskRunResponse>(`/tasks/${taskId}/runs/${runId}`, { retry: 3, dedupe: "cancel" })
        },
        async fetchTaskRunLogs(taskId: string, runId: string, offset: number = 0) {
            const $fetch = await useServer$fetch();
            let query: GetTaskRunLogsParams = {
                limit: 200,
                offset,
            };
            return await $fetch<GetTaskRunLogsResponse>(`/tasks/${taskId}/runs/${runId}/logs`, { method: 'get', retry: 3, query });
        },
        async createTask(task: CreateTaskCommand) {
            const $fetch = await useServer$fetch();
            return await $fetch<void>("/tasks", { method: "post", body: task })
        },
        async archiveTask(taskId: string) {
            const $fetch = await useServer$fetch();
            return await $fetch<void>(`/tasks/${taskId}/archive`, { method: "post" })
        },
        async updateTask(taskId: string, command: UpdateTaskCommand) {
            const $fetch = await useServer$fetch();
            return await $fetch<void>(`/tasks/${taskId}`, { method: "patch", body: command })
        },
        async checkTaskIdIsAvailable(taskId: string) {
            const $fetch = await useServer$fetch();
            try {
                await $fetch<void>(`/tasks/${taskId}`, { method: "head" })
                return false;
            } catch (error) {
                if (error instanceof FetchError && error.status === 404) {
                    return true;
                }
                throw error;
            }
        }
    }
}
