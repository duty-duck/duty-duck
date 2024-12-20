<template>
  <div class="task-runs-table mb-3 mt-4">
    <!-- Table header (hidden on mobile) -->
    <div class="row head-row mb-2 d-none d-lg-flex text-secondary">
      <div class="col" v-if="showColumns.includes('startedAt')">
        <Icon name="ph:calendar-duotone" aria-hidden /> {{ $t('dashboard.taskRuns.startedAt') }}
      </div>
      <div class="col" v-if="showColumns.includes('completedAt')">
        <Icon name="ph:check-circle-duotone" aria-hidden /> {{ $t('dashboard.taskRuns.completedAt') }}
      </div>
      <div class="col" v-if="showColumns.includes('taskId')">
        <Icon name="ph:task" aria-hidden /> {{ $t('dashboard.taskRuns.taskId') }}
      </div>
      <div class="col" v-if="showColumns.includes('status')">
        <Icon name="ph:circle-dashed" aria-hidden /> {{ $t('dashboard.taskRuns.status') }}
      </div>
      <div class="col" v-if="showColumns.includes('duration')">
        <Icon name="ph:clock-duotone" aria-hidden /> {{ $t('dashboard.taskRuns.duration') }}
      </div>
      <div class="col" v-if="showColumns.includes('exitCode')">
        <Icon name="ph:terminal-duotone" aria-hidden /> {{ $t('dashboard.taskRuns.exitCode') }}
      </div>
    </div>

    <!-- Table rows -->
    <NuxtLink class="card mb-3 shadow-sm slide-up-fade-in" v-for="taskRun in taskRuns" :key="taskRun.taskId"
      :to="localePath(`/dashboard/tasks/${taskRun.taskId}/runs/${taskRun.startedAt}`)">
      <div class="card-body">
        <div class="row row-gap-2">
          <!-- Started At -->
          <div class="col-lg" v-if="showColumns.includes('startedAt')">
            <label class="text-secondary d-flex align-items-center gap-1 d-lg-none">
              <Icon name="ph:clock" aria-hidden /> {{ $t('dashboard.taskRuns.startedAt') }}
            </label>
            {{ $d(new Date(taskRun.startedAt), "long") }}
          </div>

          <!-- Completed At -->
          <div class="col-lg" v-if="showColumns.includes('completedAt')">
            <label class="text-secondary d-flex align-items-center gap-1 d-lg-none">
              <Icon name="ph:check-circle" aria-hidden /> {{ $t('dashboard.taskRuns.completedAt') }}
            </label>
            <span v-if="taskRun.completedAt">{{ $d(new Date(taskRun.completedAt), "long") }}</span>
            <span v-else>--</span>
          </div>

          <!-- Task ID -->
          <div class="col-lg" v-if="showColumns.includes('taskId')">
            <label class="text-secondary d-flex align-items-center gap-1 d-lg-none">
              <Icon name="ph:task" aria-hidden /> {{ $t('dashboard.taskRuns.taskId') }}
            </label>
            {{ taskRun.taskId }}
          </div>

          <!-- Status -->
          <div class="col-lg" :class="{ 'text-danger': taskRun.status == 'failed' }"
            v-if="showColumns.includes('status')">
            <label class="d-lg-none mt-2 text-secondary d-block">
              <Icon name="ph:circle-dashed" aria-hidden /> {{ $t('dashboard.taskRuns.status') }}
            </label>
            <TaskRunStatusLabel :status="taskRun.status" />
          </div>

          <!-- Duration -->
          <div class="col-lg" v-if="showColumns.includes('duration')">
            <label class="text-secondary d-flex align-items-center gap-1  d-lg-none">
              <Icon name="ph:clock-duotone" aria-hidden /> {{ $t('dashboard.taskRuns.duration') }}
            </label>
            {{ formatDurationFromDates(taskRun.startedAt, taskRun.completedAt || now, locale) }}
          </div>

          <!-- Exit Code -->
          <div class="col-lg" v-if="showColumns.includes('exitCode')">
            <label class="text-secondary d-flex align-items-center gap-1 d-lg-none">
              <Icon name="ph:terminal-duotone" aria-hidden /> {{ $t('dashboard.taskRuns.exitCode') }}
            </label>
            {{ taskRun.exitCode }}
          </div>

        </div>
      </div>
    </NuxtLink>
  </div>
</template>

<script setup lang="ts">
import { useNow } from '@vueuse/core';
import type { TaskRun } from 'bindings/TaskRun';
import { formatDurationFromDates } from '~/utils/duration';

const { locale } = useI18n();
const localePath = useLocalePath();
const now = useNow();

type Column = "startedAt" | "completedAt" | "duration" | "taskId" | "status" | "exitCode"

const { taskRuns, showColumns = ["startedAt", "completedAt", "duration", "status", "exitCode"] } = defineProps<{
  taskRuns: TaskRun[],
  showColumns?: Column[]
}>()
</script>

<style lang="scss" scoped>
.card {
  text-decoration: none;
}

@for $i from 1 through 10 {
  @keyframes slideUpFadeIn#{$i} {
    0% {
      opacity: 0;
      transform: translateY(30px);
    }

    #{$i* 10 + "%"} {
      opacity: 0;
      transform: translateY(30px);
    }

    100% {
      opacity: 1;
      transform: translateY(0);
    }
  }
}

@keyframes slideUpFadeIn {
  from {
    opacity: 0;
    transform: translateY(30px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.slide-up-fade-in {
  @for $i from 1 through 10 {
    &:nth-child(#{$i}n) {
      animation: slideUpFadeIn#{$i} 0.3s ease-out;
    }
  }
}
</style>