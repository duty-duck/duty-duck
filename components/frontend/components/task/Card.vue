<script setup lang="ts">
import type { Task } from "bindings/Task";
const localePath = useLocalePath();
const taskRepository = useTasksRepository();
const { task, animated } = defineProps<{ task: Task; animated?: boolean }>();

const { data: taskRuns, refresh: refreshTaskRuns } = await taskRepository.useTaskRuns(task.id, { pageNumber: 1, itemsPerPage: 12, includeStatuses: [] });
const reversedRuns = computed(() => taskRuns.value?.runs.toReversed());

defineExpose({
  refresh: async () => {
    await refreshTaskRuns();
  }
});
</script>

<template>
  <NuxtLink :to="localePath(`/dashboard/tasks/${task.id}`)" class="card shadow-sm" style="overflow: hidden"
    :class="{ 'slide-up-fade-in': animated }">
    <BCardBody class="d-flex align-items-center py-4 px-2 overflow-hidden">
      <TaskStatusIcon :status="task.status" class="mx-4 mx-lg-5" />
      <div class="flex-grow-1">
        <div class="h6 mb-3">
          {{ task.name }}
        </div>

        <!-- Status and last run -->
        <div class="d-flex align-items-center gap-2 mb-2">
          <TaskStatusLabel :status="task.status" />
          <div class="text-secondary small" v-if="reversedRuns && reversedRuns.length > 0">
            {{ $t('dashboard.tasks.lastRunOn', { date: $d(new Date(reversedRuns[0].startedAt), 'long') }) }}
          </div>
        </div>

        <!-- Cron schedule and next due date -->
        <div class="small text-secondary d-flex align-items-center gap-2" v-if="task.cronSchedule">
          <Icon name="ph:clock" />
          {{ $t('dashboard.tasks.card.cronSchedule', { schedule: task.cronSchedule }) }}
          <span v-if="task.nextDueAt">
            &nbsp;|&nbsp;{{ $d(new Date(task.nextDueAt), 'long') }}
          </span>

        </div>
      </div>

    </BCardBody>

    <!-- Last runs grid -->
    <div class="bg-light py-2 px-3 d-flex align-items-center gap-3">
      <template v-if="reversedRuns && reversedRuns.length > 0">
        <small class="text-secondary">
          {{ $t('dashboard.tasks.lastRuns') }}
        </small>
        <TaskRunsGridChart :task-runs="reversedRuns" />
      </template>
      <template v-else>
        <small class="text-secondary">
          {{ $t('dashboard.tasks.card.noRuns') }}
        </small>
      </template>

    </div>
  </NuxtLink>
</template>

<style scoped lang="scss">
.btn-toolbar {
  display: none;
}

.card {
  cursor: pointer;
  text-decoration: inherit;
}

.card:hover {
  .btn-toolbar {
    display: unset;
  }
}

// Reusing the same animation logic
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