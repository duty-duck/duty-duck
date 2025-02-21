<script lang="ts" setup>
import type { UpdateTaskCommand } from "bindings/UpdateTaskCommand";
import type { TaskFormData } from "~/components/task/Form.vue";
import { usePermissionGrant } from "~/composables/authComposables";

await usePermissionGrant("writeTasks");

const repo = useTasksRepository();
const route = useRoute();
const localePath = useLocalePath();

const { data: taskData } = await repo.useTask(
    route.params.taskId as string
);

const onSubmit = async (task: TaskFormData) => {
    const command: UpdateTaskCommand = {
        ...task.notificationSettings,
        ...task,
    };
    await repo.updateTask(route.params.taskId as string, command);

    navigateTo(localePath(`/dashboard/tasks/${route.params.taskId}`));
};

const formData = computed<TaskFormData>(() => {
    const task = taskData.value!.task;
    return {
        id: task.userId,
        name: task.name,
        description: task.description,
        cronSchedule: task.cronSchedule,
        sheduleTimezone: task.scheduleTimezone,
        startWindowSeconds: task.startWindowSeconds,
        latenessWindowSeconds: task.latenessWindowSeconds,
        heartbeatTimeoutSeconds: task.heartbeatTimeoutSeconds,
        notificationSettings: {
            smsNotificationEnabled: task.smsNotificationEnabled,
            emailNotificationEnabled: task.emailNotificationEnabled,
            pushNotificationEnabled: task.pushNotificationEnabled,
        },
        metadata: task.metadata,
    };
});
</script>
<template>
    <BContainer>
        <BBreadcrumb>
            <BBreadcrumbItem :to="localePath('/dashboard')">{{
                $t("dashboard.mainSidebar.home")
                }}</BBreadcrumbItem>
            <BBreadcrumbItem :to="localePath('/dashboard/tasks')">{{
                $t("dashboard.mainSidebar.tasks")
                }}</BBreadcrumbItem>
            <BBreadcrumbItem active>
                {{ $t("dashboard.tasks.createTaskTitle") }}
            </BBreadcrumbItem>
        </BBreadcrumb>
        <h2 class="mb-3">{{ $t("dashboard.tasks.createTaskTitle") }}</h2>
        <TaskForm :data="formData" @submit="onSubmit" />
    </BContainer>
</template>
