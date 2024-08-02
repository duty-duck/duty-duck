<script lang="ts" setup>
import { refDebounced, useDebounceFn, useIntervalFn } from "@vueuse/core";
import type { HttpMonitorStatus } from "bindings/HttpMonitorStatus";
import { allStatuses } from "~/components/MonitorStatusDropdown.vue";

const route = useRoute();
const router = useRouter();
const query = computed(() => route.query.query as string || '');
const queryDebounced = refDebounced(query, 250);
const pageNumber = computed(() => route.query.page ? Number(route.query.page) : 1);
const includeStatuses = computed(() => route.query.statuses && route.query.statuses.length ? route.query.statuses as HttpMonitorStatus[] : allStatuses);

const fetchParams = computed(() => ({
  pageNumber: pageNumber.value,
  include: includeStatuses.value,
  query: queryDebounced.value,
  itemsPerPage: 10,
}));

const repository = useHttpMonitorRepository();
const { status, data, refresh } = await repository.useHttpMonitors(fetchParams);

const onPageChange = (page: number) => {
  router.push({ path: "/dashboard/monitors", query: { page, statuses: includeStatuses.value, query: query.value } })
}
const onQueryChange = (event: KeyboardEvent) => {
  router.push({ path: "/dashboard/monitors", query: { page: pageNumber.value, statuses: includeStatuses.value, query: (event.target as any).value } })
}
const onIncludeStatusChange = (statuses: HttpMonitorStatus[]) => {
  router.push({ path: "/dashboard/monitors", query: { pageNumber: pageNumber.value, query: query.value, statuses } })
}
const onClearFilters = () => {
  router.push({ path: "/dashboard/monitors", query: { pageNumber: pageNumber.value, query: "", statuses: [] } })
}

const hiddenMonitorsCount = computed(() => {
  if (!data.value) {
    return 0
  }
  return data.value!.totalNumberOfResults - data.value!.totalNumberOfFilteredResults
})

if (data.value?.items.length == 0 && pageNumber.value > 1) {
  router.replace("/dasboard/monitors");
}

useIntervalFn(() => {
  refresh();
}, 10000);
</script>

<template>
  <div>
    <BBreadcrumb>
      <BBreadcrumbItem to="/dashboard">Home</BBreadcrumbItem>
      <BBreadcrumbItem active>Monitors</BBreadcrumbItem>
    </BBreadcrumb>
    <div class="d-flex align-items-center justify-content-between">
      <h2>Monitors</h2>
      <AddHttpMonitorButton />
    </div>
    <div class="small text-secondary mb-2">
      {{ data?.totalNumberOfResults }} Total Monitors, 10 items per page
      <span v-if="hiddenMonitorsCount != 0">, {{ hiddenMonitorsCount }} monitors are not shown because of
        filtering</span>
    </div>
    <div class="d-flex flex-column flex-md-row gap-2 mb-4 filtering-bar">
      <MonitorStatusDropdown :model-value="includeStatuses" @update:model-value="onIncludeStatusChange" />
      <BInput class="border border-secondary bg-transparent" :model-value="query" @input="onQueryChange"
        placeholder="Search by URL" />
      <BButton class="flex-shrink-0 icon-link" variant="outline-secondary" @click="onClearFilters">
        <Icon name="ph:x-square-fill" />
        Clear filters
      </BButton>
    </div>
    <BAlert variant="danger" :model-value="status == 'error'">
      Failed to fetch HTTP monitors from the server. Please try again.
    </BAlert>
    <div v-if="data?.totalNumberOfResults == 0" class="text-secondary text-center my-5">
      <Icon name="ph:pulse-duotone" size="120px" />
      <h3>Nothing here yet</h3>
      <p class="lead">
        Create your first monitor to start monitoring your website
      </p>
      <AddHttpMonitorButton class="m-3" />
    </div>
    <MonitorCard v-for="monitor in data?.items" :key="monitor.id" v-bind="monitor" />
    <BPagination :model-value="pageNumber" @update:modelValue="onPageChange"
      :total-rows="data?.totalNumberOfFilteredResults" :per-page="10" prev-text="Prev" next-text="Next" />
  </div>
</template>

<style>
.filtering-bar {
  position: sticky;
  top: 60px;
  z-index: 1;
  padding: 10px 0;
  backdrop-filter: blur(20px);
  background-color: rgba(248, 249, 250, 0.8);
  
}
</style>