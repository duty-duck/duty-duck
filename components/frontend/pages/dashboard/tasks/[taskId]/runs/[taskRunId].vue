<script setup lang="ts">
definePageMeta({
    permissions: ['readTaskRuns']
});

const { params: { taskId, taskRunId } } = useRoute();
const localePath = useLocalePath();
const { t, d, locale } = useI18n();

const taskRepo = useTasksRepository();
const { data: taskRes } = await taskRepo.useTask(taskId as string);
const { data: runRes, refresh } = await taskRepo.useTaskRun(taskId as string, taskRunId as string);

// setup periodic refresh
// refresh the task run periodically until it is completed
useDataRefreshInterval(() => {
    if (!runRes.value?.taskRun.completedAt) {
        refresh()
    }
});

const detailsColumns = computed(() => {
    const taskRun = runRes.value?.taskRun!;
    return [
        {
            icon: 'ph:calendar',
            label: t("dashboard.taskRuns.startedAt"),
            value: d(new Date(taskRun.startedAt), "verbose")
        },
        ...taskRun.completedAt ? [{
            icon: 'ph:calendar',
            label: t("dashboard.taskRuns.completedAt"),
            value: d(new Date(taskRun.completedAt), "verbose")
        },
        {
            icon: 'ph:clock',
            label: t("dashboard.taskRuns.duration"),
            value: formatDurationFromDates(taskRun.startedAt, taskRun.completedAt, locale.value)
        },
        {
            icon: 'ph:terminal-duotone',
            label: t("dashboard.taskRuns.exitCode"),
            value: taskRun.exitCode
        },

        ] : [],
        ...taskRun.exitCode ? [
            {
                icon: 'ph:terminal-duotone',
                label: t("dashboard.taskRuns.exitCode"),
                value: taskRun.exitCode
            },
        ] : []
    ]
})
</script>

<template>

    <BContainer v-if="taskRes && runRes">
        <BBreadcrumb>
            <BBreadcrumbItem :to="localePath('/dashboard')">{{ $t("dashboard.mainSidebar.home") }}</BBreadcrumbItem>
            <BBreadcrumbItem :to="localePath('/dashboard/tasks')">{{ $t("dashboard.mainSidebar.tasks") }}
            </BBreadcrumbItem>
            <BBreadcrumbItem active>{{ taskRes.task.name }}</BBreadcrumbItem>
        </BBreadcrumb>

        <!-- Top section (task run name and status )-->
        <section class="my-5">

            <h1 class="h2">{{ taskRes.task.name }}</h1>
            <h2 class="h5 text-secondary d-flex gap-2 align-items-end">
                <Icon name="ph:calendar" />
                {{ $t("dashboard.taskRuns.runTitle", { date: $d(new Date(runRes.taskRun.startedAt), "long") }) }}
            </h2>
            <TaskRunStatusLabel :status="runRes.taskRun.status" />
        </section>

        <!-- Details section -->
        <section class="mb-5">
            <BRow :cols-sm="2" :cols-md="3" :cols-lg="4" :cols-xl="5" gutter-x="2" gutter-y="2"
                class="equal-height-row ">
                <BCol v-for="col in detailsColumns">
                    <BCard class="h-100">
                        <h6 class="d-flex align-items-end gap-1">
                            <Icon :name="col.icon" />
                            {{ col.label }}
                        </h6>
                        {{ col.value }}
                    </BCard>
                </BCol>

            </BRow>
        </section>

        <!-- Logs section -->
        <section>
            <h3 class="mb-4">{{ $t('dashboard.taskRuns.logs.title') }}</h3>
            <LazyTaskRunLogViewer :task-id="taskId as string" :task-run-id="taskRunId as string"
                :task-run-is-active="taskRes.task.status == 'running'" />
        </section>
    </BContainer>
</template>