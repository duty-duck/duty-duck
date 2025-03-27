<script setup lang="ts">
import { refDebounced } from '@vueuse/core';
import { useRouteQuery } from '@vueuse/router';
import type { ListTasksParams } from 'bindings/ListTasksParams';
import type { TaskStatus } from 'bindings/TaskStatus';
import { allStatuses } from '~/components/task/StatusDropdown.vue';

definePageMeta({
  permissions: ['readTasks']
});

const taskRepository = useTasksRepository();
const query = useRouteQuery("query", "");
const queryDebounced = refDebounced(query, 250);
const pageNumber = useRouteQuery("pageNumber", 1, { transform: Number });
const includeStatuses = useRouteQuery<TaskStatus[]>("statuses", ["failing", "healthy", "late", "running", "due", "absent"]);
const { data: metadataFilter, clear: clearMetadataFilter } = useMetadataFilterQuery();
const localePath = useLocalePath();
const showMetadataOffcanvas = useBoolQueryParam("metadataOpen");

const listTasksParams = computed<ListTasksParams>(() => ({
  pageNumber: pageNumber.value,
  itemsPerPage: 10,
  include: includeStatuses.value,
  query: queryDebounced.value,
  metadataFilter: metadataFilter.value,
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

const { data: filterableMetadataFields } = await taskRepository.useFilterableMetadataFields();

// Every 10 seconds, refresh the tasks
useDataRefreshInterval(refreshTasks);
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
    <TaskFilteringBar v-model:includeStatuses="includeStatuses" v-model:query="query" @clear-filters="onClearFilters"
      @toggle-metadata="showMetadataOffcanvas = true" :metadata-filter="metadataFilter" />
    <div class="d-grid row-gap-3 mt-3" v-if="tasks?.items.length">
      <TaskCard animated v-for="t in tasks?.items" :task="t" :key="t.id" />
      <BPagination v-if="tasks?.totalNumberOfFilteredResults! > 10" v-model="pageNumber"
        :prev-text="$t('pagination.prev')" :next-text="$t('pagination.next')"
        :total-rows="tasks?.totalNumberOfFilteredResults" :per-page="10" />
    </div>
    <div v-else-if="tasks?.totalNumberOfResults == 0" class="text-secondary text-center my-5">
      <Icon name="ph:pulse-duotone" size="120px" />
      <h3>{{ $t("dashboard.tasks.emptyPage.title") }}</h3>
      <p class="lead">
        {{ $t("dashboard.tasks.emptyPage.text") }}
      </p>
    </div>
    <div v-else-if="tasks?.totalNumberOfFilteredResults == 0" class="text-secondary text-center my-5">
      <Icon name="ph:pulse-duotone" size="120px" />
      <h3>{{ $t("dashboard.tasks.noResults.title") }}</h3>
      <p class="lead">
        {{ $t("dashboard.tasks.noResults.text") }}
      </p>
      <BButton variant="outline-secondary" @click="onClearFilters">{{ $t("dashboard.tasks.clearFilters") }}
      </BButton>
    </div>
    <div class="mt-5 text-center">
      <h4 class="fs-6 text-muted">{{ $t("dashboard.tasks.cliCommandLabel") }}</h4>
      <code>
        dutyduck tasks run --create --task-id "db-backup" pgbackrest --stanza=main backup
      </code>
    </div>

    <!-- metadata filter offcanvas -->
    <BOffcanvas v-model="showMetadataOffcanvas" placement="end" body-class="p-0">
      <template #title>
        <h6 class="d-flex align-items-center gap-2 mb-0">
          <Icon name="ph:funnel" aria-hidden />
          {{ $t('dashboard.facets.title') }}
        </h6>
      </template>
      <DashboardMetadataFacets v-model="metadataFilter" :metadata="filterableMetadataFields!" />
    </BOffcanvas>
  </BContainer>
</template>
