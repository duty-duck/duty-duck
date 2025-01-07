<script setup lang="ts">
import { useNow } from '@vueuse/core';
import { useRouteQuery } from '@vueuse/router';

import cronstrue from 'cronstrue';
import 'cronstrue/locales/fr';

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
const humanReadableCron = computed(() => {
  return taskResponse.value?.task.cronSchedule ? cronstrue.toString(taskResponse.value.task.cronSchedule, { locale: locale.value, use24HourTimeFormat: true, verbose: true }) : '';
});

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
    <div class="row mb-5 row-gap-3 r">
      <!-- Last status change column -->
      <div class="col-md-4">
        <BCard class="h-100">
          <p>{{ $t("dashboard.tasks.lastStatusChange") }}</p>
          <template v-if="lastStatusChange">
            <div class="text-muted mb-2">
              {{ $d(new Date(taskResponse.task.lastStatusChangeAt!), "long") }}
            </div>
            <p class="h4">
              {{ $t("dashboard.tasks.dateAgo", {
                date: lastStatusChange,
              })
              }}
            </p>
            <template v-if="taskResponse.task.previousStatus">
              <hr>
              <div class="text-muted small d-flex align-items-center gap-2">
                {{ $t("dashboard.tasks.previousStatus") }}
                <TaskStatusLabel :status="taskResponse.task.previousStatus" />
              </div>
            </template>
          </template>
          <p class="text-muted" v-else>
            --
          </p>
        </BCard>
      </div>
      <!-- Next due at column -->
      <div class="col-md-4">
        <BCard class="h-100" :class="{ 'bg-light': !taskResponse.task.cronSchedule }">
          <template v-if="taskResponse.task.cronSchedule">
            <template v-if="taskResponse.task.status === 'late' || taskResponse.task.status === 'absent'">
              <p>{{ $t("dashboard.tasks.initiallyDueOn") }}</p>
              <p class="h4" v-if="taskResponse.task.nextDueAt">
                {{ $d(new Date(taskResponse.task.nextDueAt), "long") }}
              </p>
            </template>
            <template v-else>
              <p>{{ $t("dashboard.tasks.nextDueOn") }}</p>
              <p class="h4" v-if="taskResponse.task.nextDueAt">
                {{ $d(new Date(taskResponse.task.nextDueAt), "long") }}
              </p>
            </template>
            <hr>
            <div class="small text-muted">
              {{ $t("dashboard.tasks.schedule") }} {{ taskResponse.task.cronSchedule }}<br>({{
                humanReadableCron }})
            </div>
          </template>
          <template v-else>
            <p>{{ $t("dashboard.tasks.schedule") }}</p>
            <p class="text-muted">
              {{ $t("dashboard.tasks.notAScheduledTask") }}
            </p>
          </template>

        </BCard>
      </div>
    </div>

    <!-- Task description -->
    <section class="mb-5" v-if="taskResponse.task.description">
      <h3 class="fs-5 d-flex align-items-center gap-2">
        <Icon name="ph:info" />
        {{ $t("dashboard.tasks.description") }}
      </h3>
      <p class="text-muted">
        {{ taskResponse.task.description }}
      </p>
    </section>

    <!-- Task runs -->
    <section class="d-flex flex-column gap-3">
      <h3 class="fs-5 d-flex align-items-center gap-2">
        <Icon name="ph:clock-counter-clockwise" />
        {{ $t("dashboard.tasks.runsHistory") }}
      </h3>
      <div v-if="taskRunsResponse?.runs && taskRunsResponse.runs.length > 0">
        <TaskRunTableView :taskRuns="taskRunsResponse.runs" />
        <BPagination v-model="pageNumber" :total-rows="taskRunsResponse.totalFilteredRuns"
          :limit="taskRunsParams.itemsPerPage" :next-text="$t('pagination.next')" :prev-text="$t('pagination.prev')"
          pills />
      </div>
      <div class="text-center text-muted" v-else>
        {{ $t("dashboard.tasks.noRuns") }}
      </div>

      <!-- Start run command help -->
      <div class="text-center text-muted">
        {{ $t("dashboard.tasks.startRunCommandCta") }}
        <br />
        <code>{{ $t("dashboard.tasks.startRunCommand", { taskId: taskResponse.task.id }) }}</code>
      </div>
    </section>
  </BContainer>
</template>