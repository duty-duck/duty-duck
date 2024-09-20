<script lang="ts" setup>
import { refDebounced, useIntervalFn } from "@vueuse/core";
import type { HttpMonitorStatus } from "bindings/HttpMonitorStatus";
import { allStatuses } from "~/components/httpMonitor/StatusDropdown.vue";

ensurePemissionOnBeforeMount("readHttpMonitors");

const localePath = useLocalePath();
const path = localePath("/dashboard/httpMonitors");
const route = useRoute();
const router = useRouter();
const query = computed(() => (route.query.query as string) || "");
const queryDebounced = refDebounced(query, 250);
const pageNumber = computed({
  get() {
    return route.query.page ? Number(route.query.page) : 1
  },
  set(value: number) {
    router.push({
      path,
      query: { page: value, statuses: includeStatuses.value, query: query.value },
    });
  }
})
const includeStatuses = computed(() =>
  route.query.statuses && route.query.statuses.length
    ? (route.query.statuses as HttpMonitorStatus[])
    : allStatuses
);

const fetchParams = computed(() => ({
  pageNumber: pageNumber.value,
  include: includeStatuses.value,
  query: queryDebounced.value,
  itemsPerPage: 10,
}));

const repository = useHttpMonitorRepository();
const { status, data, refresh } = await repository.useHttpMonitors(fetchParams);


const onQueryChange = (event: Event) => {
  router.push({
    path,
    query: {
      page: pageNumber.value,
      statuses: includeStatuses.value,
      query: (event.target as HTMLInputElement).value,
    },
  });
};
const onIncludeStatusChange = (statuses: HttpMonitorStatus[]) => {
  router.push({
    path,
    query: { pageNumber: pageNumber.value, query: query.value, statuses },
  });
};
const onClearFilters = () => {
  router.push({
    path,
    query: { pageNumber: pageNumber.value, query: "", statuses: [] },
  });
};

const hiddenMonitorsCount = computed(() => {
  if (!data.value) {
    return 0;
  }
  return (
    data.value!.totalNumberOfResults - data.value!.totalNumberOfFilteredResults
  );
});

if (data.value?.items.length == 0 && pageNumber.value > 1) {
  router.replace(path);
}

useIntervalFn(() => {
  refresh();
}, 10000);
</script>

<template>
  <div>
    <BContainer>
      <BBreadcrumb>
        <BBreadcrumbItem to="/dashboard">{{
          $t("dashboard.mainSidebar.home")
        }}</BBreadcrumbItem>
        <BBreadcrumbItem active>{{
          $t("dashboard.sidebar.monitors")
        }}</BBreadcrumbItem>
      </BBreadcrumb>
      <div class="d-flex align-items-center justify-content-between">
        <h2>{{ $t("dashboard.monitors.pageTitle") }}</h2>
        <HttpMonitorAddButton />
      </div>
      <div class="small text-secondary mb-2">
        {{
          $t(
            "dashboard.monitors.totalMonitorCount",
            data?.totalNumberOfResults || 0
          )
        }}, {{ $t("dashboard.monitors.itemsPerPage", 10) }}
        <span v-if="hiddenMonitorsCount != 0">
          ,
          {{
            $t("dashboard.monitors.filteredMonitorCount", hiddenMonitorsCount)
          }}
        </span>
      </div>
    </BContainer>
    <div v-if="data?.totalNumberOfResults == 0" class="text-secondary text-center my-5">
      <Icon name="ph:pulse-duotone" size="120px" />
      <h3>{{ $t("dashboard.monitors.emptyPage.title") }}</h3>
      <p class="lead">
        {{ $t("dashboard.monitors.emptyPage.text") }}
      </p>
      <HttpMonitorAddButton class="m-3" />
    </div>
    <BContainer v-else class="d-grid row-gap-3">
      <HttpMonitorFilteringBar :include-statuses="includeStatuses" :query="query"
        @update-include-statuses="onIncludeStatusChange" @update-query="onQueryChange"
        @clear-filters="onClearFilters" />
      <HttpMonitorCard v-for="monitor in data?.items" :key="monitor.id" :monitor="monitor" animated />
      <BPagination v-if="data?.totalNumberOfFilteredResults! > 10" v-model="pageNumber"
        :prev-text="$t('pagination.prev')" :next-text="$t('pagination.next')"
        :total-rows="data?.totalNumberOfFilteredResults" :per-page="10" />
    </BContainer>
  </div>
</template>
