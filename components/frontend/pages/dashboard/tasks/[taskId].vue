<script setup lang="ts">
import { useNow } from '@vueuse/core';
import { useRouteQuery } from '@vueuse/router';

const { locale } = useI18n();
const localePath = useLocalePath();
const { params: { taskId } } = useRoute();
const taskRepo = useTasksRepository();
const now = useNow();

const pageNumber = useRouteQuery("pageNumber", 1, { transform: Number });
const taskRunsParams = computed(() => ({ itemsPerPage: 10, pageNumber: pageNumber.value, includeStatuses: null }));

const { data: taskResponse } = await taskRepo.useTask(taskId as string);
const { data: lastTaskRunResponse } = await taskRepo.useTaskRuns(taskId as string, { itemsPerPage: 1, pageNumber: 1, includeStatuses: null });
const { data: taskRunsResponse } = await taskRepo.useTaskRuns(taskId as string, taskRunsParams);

const lastTaskRun = computed(() => lastTaskRunResponse.value?.runs?.[0]);

const lastStatusChange = computed(() => {
    if (!taskResponse.value?.task.lastStatusChangeAt) {
        return null;
    }
    const duration =
        now.value.getTime() -
        new Date(taskResponse.value.task.lastStatusChangeAt).getTime();

    return formatDuration(duration, locale.value);
});
</script>

<template>
    <BContainer v-if="taskResponse">
        <BBreadcrumb>
            <BBreadcrumbItem :to="localePath('/dashboard')">{{ $t("dashboard.mainSidebar.home") }}</BBreadcrumbItem>
            <BBreadcrumbItem :to="localePath('/dashboard/tasks')">{{ $t("dashboard.mainSidebar.tasks") }}
            </BBreadcrumbItem>
            <BBreadcrumbItem active>{{ taskResponse.task.name }}</BBreadcrumbItem>
        </BBreadcrumb>

        <!-- Task name and status -->
        <div class="my-5 py-3">
            <h2 class="h4">
                {{ taskResponse.task.name }}
            </h2>
            <TaskStatusLabel :status="taskResponse.task.status" />
            &nbsp;
            <span v-if="lastTaskRun" class="small text-secondary">
                {{ $t("dashboard.tasks.lastRunOn", { date: $d(new Date(lastTaskRun.startedAt!), "long") }) }}
            </span>
        </div>

        <!-- Task overview -->
        <div class="row mb-5  row-gap-3">
            <div class="col-md-4">
                <BCard>
                    <p>{{ $t("dashboard.tasks.lastStatusChange") }}</p>
                    <p class="h4">
                        {{
                            lastStatusChange
                                ? $t("dashboard.tasks.dateAgo", {
                                    date: lastStatusChange,
                                })
                        : "--"
                        }}
                    </p>
                </BCard>
            </div>
            <div class="col-md-4">
                <BCard :class="{ 'bg-light': !taskResponse.task.cronSchedule }">
                    <p>{{ $t("dashboard.tasks.schedule") }}</p>
                    <p class="h4" v-if="taskResponse.task.cronSchedule">
                        {{ taskResponse.task.cronSchedule }}
                    </p>
                    <p class="text-muted" v-else>
                        {{ $t("dashboard.tasks.notAScheduledTask") }}
                    </p>
                </BCard>
            </div>
            <div class="col-md-4">
                <BCard :class="{ 'bg-light': !taskResponse.task.nextDueAt }">
                    <p>{{ $t("dashboard.tasks.nextDueAt") }}</p>
                    <p class="h4" v-if="taskResponse.task.nextDueAt">
                        {{ $d(new Date(taskResponse.task.nextDueAt), "long") }}
                    </p>
                    <p class="text-muted" v-else>
                        {{ $t("dashboard.tasks.notAScheduledTask") }}
                    </p>
                </BCard>
            </div>
        </div>

        <!-- Task runs -->
        <section class="d-flex flex-column gap-3">
            <h3 class="fs-5">{{ $t("dashboard.tasks.runsHistory") }}</h3>
            <TaskRunTableView v-if="taskRunsResponse?.runs" :taskRuns="taskRunsResponse.runs" />
        </section>
    </BContainer>
</template>