<script lang="ts" setup>
import type { TaskFormData } from "~/components/task/Form.vue";
import { usePermissionGrant } from "~/composables/authComposables";

await usePermissionGrant("writeTasks");

const localePath = useLocalePath();
const tasksRepository = useTasksRepository();

const onSubmit = async (data: TaskFormData) => {
    await tasksRepository.createTask({
        ...data,
        ...data.notificationSettings,
    });
    navigateTo(localePath("/dashboard/tasks"));
};
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
        <TaskForm @submit="onSubmit" />
    </BContainer>
</template>
