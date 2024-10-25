<script lang="ts" setup>
import { refDebounced, useIntervalFn } from "@vueuse/core";
import { useRouteQuery } from "@vueuse/router";
import type { HttpMonitorStatus } from "bindings/HttpMonitorStatus";
import { allStatuses } from "~/components/httpMonitor/StatusDropdown.vue";
import { usePermissionGrant } from "~/composables/authComposables";

await usePermissionGrant("readHttpMonitors");

const localePath = useLocalePath();
const query = useRouteQuery("query", "");
const queryDebounced = refDebounced(query, 250);
const pageNumber = useRouteQuery("pageNumber", 1, { transform: Number });
const includeStatuses = useRouteQuery("statuses", allStatuses);

const fetchParams = computed(() => ({
  pageNumber: pageNumber.value,
  include: includeStatuses.value,
  query: queryDebounced.value,
  itemsPerPage: 10,
}));

const repository = await useHttpMonitorRepository();
const { status, data, refresh } = await repository.useHttpMonitors(fetchParams);


const onQueryChange = (event: Event) => {
  navigateTo({
    query: {
      page: pageNumber.value,
      statuses: includeStatuses.value,
      query: (event.target as HTMLInputElement).value,
    },
  });
};
const onIncludeStatusChange = (statuses: HttpMonitorStatus[]) => {
  navigateTo({
    query: { pageNumber: pageNumber.value, query: query.value, statuses },
  });
};
const onClearFilters = () => {
  navigateTo({
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
  navigateTo({ query: { pageNumber: 1 } });
}

useIntervalFn(() => {
  refresh();
}, 10000);
</script>

<template>
  <div>
    <BContainer>
      <BBreadcrumb>
        <BBreadcrumbItem :to="localePath('/dashboard')">{{
          $t("dashboard.mainSidebar.home")
        }}</BBreadcrumbItem>
        <BBreadcrumbItem active>{{
          $t("dashboard.mainSidebar.monitors")
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
    <div v-else-if="data?.totalNumberOfFilteredResults == 0" class="text-secondary text-center my-5">
      <Icon name="ph:seal-check-duotone" size="120px" />
      <h3>{{ $t("dashboard.monitors.noResults.title") }}</h3>
      <p class="lead">
        {{ $t("dashboard.monitors.noResults.text") }}
      </p>
      <BButton variant="outline-secondary" @click="onClearFilters">{{ $t("dashboard.monitors.clearFilters") }}</BButton>
    </div>
    <BContainer v-else class="d-grid row-gap-3">
      <HttpMonitorFilteringBar v-model:includeStatuses="includeStatuses" v-model:query="query" @clear-filters="onClearFilters" />
      <HttpMonitorCard v-for="monitor in data?.items" :key="monitor.id" :monitor="monitor" animated />
      <BPagination v-if="data?.totalNumberOfFilteredResults! > 10" v-model="pageNumber"
        :prev-text="$t('pagination.prev')" :next-text="$t('pagination.next')"
        :total-rows="data?.totalNumberOfFilteredResults" :per-page="10" />
    </BContainer>
  </div>
</template>
