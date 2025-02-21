<script lang="ts" setup>
import { useIntervalFn } from "@vueuse/core";
import { usePermissionGrant } from "~/composables/authComposables";

await usePermissionGrant("readHttpMonitors");

const localePath = useLocalePath();
const showFacetsOffcanvas = ref(false);
const { clearFilters, listMonitorsParams, query, metadataFilter, includeStatuses, pageNumber } = await useHttpMonitorsFilters();

const repository = useHttpMonitorRepository();
const { data, refresh } = await repository.useHttpMonitors(listMonitorsParams);
const { data: filterableMetadataFields } = await repository.useFilterableMetadataFields();

const hiddenMonitorsCount = computed(() => {
  if (!data.value) {
    return 0;
  }
  return (
    data.value!.totalNumberOfResults - data.value!.totalNumberOfFilteredResults
  );
});

watchEffect(() => {
  if (data.value?.items.length == 0 && pageNumber.value > 1) {
    clearFilters()
  }
})

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
      @clear-filters="clearFilters" @toggle-metadata="showFacetsOffcanvas = true" :metadata-filter="metadataFilter" />

    <div v-if="data?.totalNumberOfResults == 0" class="text-secondary text-center my-5">
      <Icon name="ph:globe-duotone" size="120px" />
      <h3>{{ $t("dashboard.monitors.emptyPage.title") }}</h3>
      <p class="lead">
        {{ $t("dashboard.monitors.emptyPage.text") }}
      </p>
      <HttpMonitorAddButton class="m-3" />
    </div>
    <div v-else-if="data?.totalNumberOfFilteredResults == 0" class="text-secondary text-center my-5">
      <Icon name="ph:globe-duotone" size="120px" />
      <h3>{{ $t("dashboard.monitors.noResults.title") }}</h3>
      <p class="lead">
        {{ $t("dashboard.monitors.noResults.text") }}
      </p>
      <BButton variant="outline-secondary" @click="clearFilters">{{ $t("dashboard.monitors.clearFilters") }}
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
