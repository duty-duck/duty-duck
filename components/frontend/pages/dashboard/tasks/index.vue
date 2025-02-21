<script setup lang="ts">
import type { TaskCard } from '#build/components';
import { refDebounced, useIntervalFn } from '@vueuse/core';
import { useRouteQuery } from '@vueuse/router';
import type { ListTasksParams } from 'bindings/ListTasksParams';
import type { TaskStatus } from 'bindings/TaskStatus';
import { allStatuses } from '~/components/task/StatusDropdown.vue';

const taskRepository = useTasksRepository();
const query = useRouteQuery("query", "");
const queryDebounced = refDebounced(query, 250);
const pageNumber = useRouteQuery("pageNumber", 1, { transform: Number });
const includeStatuses = useRouteQuery<TaskStatus[]>("statuses", ["failing", "healthy", "late", "running", "due", "absent"]);
const localePath = useLocalePath();

const cards = ref<InstanceType<typeof TaskCard>[]>([]);

const listTasksParams = computed<ListTasksParams>(() => ({
  pageNumber: pageNumber.value,
  itemsPerPage: 10,
  include: includeStatuses.value,
  searchQuery: queryDebounced.value,
}));

const onClearFilters = () => {
  includeStatuses.value = allStatuses;
  query.value = "";
};

const { data: tasks, refresh: refreshTasks } = await taskRepository.useTasks(listTasksParams);
const hiddenTasksCount = computed(() => {
  if (!tasks.value) {
    return 0;
  }
  return (
    tasks.value!.totalNumberOfResults - tasks.value!.totalNumberOfFilteredResults
  );
});

// Every 10 seconds, refresh the tasks, and then for each rendered task card, refresh the task runs
useIntervalFn(() => {
  refreshTasks();
  cards.value.forEach(c => {
    if (c.refresh) {
      c.refresh();
    }
  });
}, 10000);
</script>

<template>
  <BContainer>
    <BBreadcrumb>
      <BBreadcrumbItem :to="localePath('/dashboard')">{{
        $t("dashboard.mainSidebar.home")
        }}</BBreadcrumbItem>
      <BBreadcrumbItem active>{{
        $t("dashboard.mainSidebar.tasks")
        }}</BBreadcrumbItem>
    </BBreadcrumb>
    <div class="d-flex align-items-center justify-content-between">
      <h2>{{ $t("dashboard.tasks.pageTitle") }}</h2>
      <TaskAddButton />
    </div>
    <div class="small text-secondary mb-2">
      {{
        $t(
          "dashboard.tasks.totalTaskCount",
          tasks?.totalNumberOfResults || 0
        )
      }}, {{ $t("dashboard.tasks.itemsPerPage", 10) }}
      <span v-if="hiddenTasksCount != 0">
        ,
        {{
          $t("dashboard.tasks.filteredTaskCount", hiddenTasksCount)
        }}
      </span>
    </div>
    <TaskFilteringBar v-model:includeStatuses="includeStatuses" v-model:query="query" @clear-filters="onClearFilters" />
    <div class="d-grid row-gap-3 mt-3">
      <TaskCard animated v-for="t in tasks?.items" :task="t" :key="t.id" ref="cards" />
      <BPagination v-if="tasks?.totalNumberOfFilteredResults! > 10" v-model="pageNumber"
        :prev-text="$t('pagination.prev')" :next-text="$t('pagination.next')"
        :total-rows="tasks?.totalNumberOfFilteredResults" :per-page="10" />
    </div>
    <div class="mt-5 text-center">
      <h4 class="fs-6 text-muted">{{ $t("dashboard.tasks.cliCommandLabel") }}</h4>
      <code>
        dutyduck tasks run --create --task-id "db-backup" pgbackrest --stanza=main backup
      </code>
    </div>
  </BContainer>
</template>
