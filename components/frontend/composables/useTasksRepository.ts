import type { UseFetchOptions } from "#app"
import type { CreateTaskCommand } from "bindings/CreateTaskCommand"
import type { GetTaskResponse } from "bindings/GetTaskResponse"
import type { ListTaskRunsParams } from "bindings/ListTaskRunsParams"
import type { ListTaskRunsResponse } from "bindings/ListTaskRunsResponse"
import type { ListTasksParams } from "bindings/ListTasksParams"
import type { ListTasksResponse } from "bindings/ListTasksResponse"
import type { UpdateTaskCommand } from "bindings/UpdateTaskCommand"
import { FetchError } from "ofetch"

export const useTasksRepository = () => {
    return {
        async useTasks(params: ListTasksParams | Ref<ListTasksParams>, opts?: UseFetchOptions<ListTasksResponse>) {
            return useServerFetch<ListTasksResponse>("/tasks", { query: params, retry: 3, dedupe: "cancel", ...(opts || {}) })
        },
        async useTaskRuns(taskId: string, params?: ListTaskRunsParams | Ref<ListTaskRunsParams>, opts?: UseFetchOptions<ListTaskRunsResponse>) {
            return useServerFetch<ListTaskRunsResponse>(`/tasks/${taskId}/runs`, { query: params, retry: 3, dedupe: "cancel", ...(opts || {}) })
        },
        async useTask(taskId: string) {
            return useServerFetch<GetTaskResponse>(`/tasks/${taskId}`, { retry: 3, dedupe: "cancel" })
        },
        async createTask(task: CreateTaskCommand) {
            const $fetch = await useServer$fetch();
            return await $fetch<void>("/tasks", { method: "post", body: task })
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
