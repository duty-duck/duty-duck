<script lang="ts" setup>
import { refDebounced, useIntervalFn } from "@vueuse/core";
import { useRouteQuery } from "@vueuse/router";
import { allStatuses } from "~/components/httpMonitor/StatusDropdown.vue";
import { usePermissionGrant } from "~/composables/authComposables";

await usePermissionGrant("readHttpMonitors");

const localePath = useLocalePath();
const query = useRouteQuery("query", "");
const queryDebounced = refDebounced(query, 250);
const pageNumber = useRouteQuery("pageNumber", 1, { transform: Number });
const includeStatuses = useRouteQuery("statuses", ['up', 'down', 'suspicious', 'recovering']);
const showFacetsOffcanvas = ref(false);
const { data: metadataFilter, clear: clearMetadataFilter } = useMetadataFilterQuery();

const fetchParams = computed(() => ({
  pageNumber: pageNumber.value,
  include: includeStatuses.value,
  query: queryDebounced.value,
  itemsPerPage: 10,
  metadataFilter: metadataFilter.value,
}));

const repository = useHttpMonitorRepository();
const { status, data, refresh } = await repository.useHttpMonitors(fetchParams);
const { data: filterableMetadataFields } = await repository.useFilterableMetadataFields();

const onClearFilters = () => {
  clearMetadataFilter();
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
    <HttpMonitorFilteringBar v-model:includeStatuses="includeStatuses" v-model:query="query"
      v-model:metadataFilter="metadataFilter" :filterableMetadataFields="filterableMetadataFields!"
      @clear-filters="onClearFilters">
      <template #default>
        <BButton variant="outline-secondary" class="d-flex align-items-center gap-1"
          @click="showFacetsOffcanvas = true">
          <Icon name="ph:funnel" aria-hidden size="1.3rem" />
          {{ $t('dashboard.facets.title') }}
        </BButton>
      </template>
    </HttpMonitorFilteringBar>
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
      <BButton variant="outline-secondary" @click="onClearFilters">{{ $t("dashboard.monitors.clearFilters") }}
      </BButton>
    </div>
    <div v-else class="d-grid row-gap-3 mt-3">
      <HttpMonitorCard v-for="monitor in data?.items" :key="monitor.id" :monitor="monitor" animated />
      <BPagination v-if="data?.totalNumberOfFilteredResults! > 10" v-model="pageNumber"
        :prev-text="$t('pagination.prev')" :next-text="$t('pagination.next')"
        :total-rows="data?.totalNumberOfFilteredResults" :per-page="10" />
    </div>
    <BOffcanvas v-model="showFacetsOffcanvas" placement="end" body-class="p-0">
      <template #header>
        <h6 class="d-flex align-items-center gap-2 mb-0">
          <Icon name="ph:funnel" aria-hidden />
          {{ $t('dashboard.facets.title') }}
        </h6>
      </template>
      <DashboardMetadataFacets v-model="metadataFilter" :metadata="filterableMetadataFields!" />
    </BOffcanvas>
  </BContainer>
</template>
