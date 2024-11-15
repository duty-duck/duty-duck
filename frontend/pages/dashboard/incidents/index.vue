<script lang="ts" setup>
import { useIntervalFn } from "@vueuse/core";
import { useRouteQuery } from "@vueuse/router";
import type { IncidentStatus } from "bindings/IncidentStatus";
import type { ListIncidentsParams } from "bindings/ListIncidentsParams";
import type { OrderDirection } from "bindings/OrderDirection";
import type { OrderIncidentsBy } from "bindings/OrderIncidentsBy";
import { allStatuses } from "~/components/incident/StatusDropdown.vue";

const localePath = useLocalePath();

const pageNumber = useRouteQuery("pageNumber", 1, { transform: Number });
const timeRange = useTimeRangeQuery();
const includeStatuses = useRouteQuery<IncidentStatus[]>("statuses", ["ongoing"]);
const orderBy = useRouteQuery<OrderIncidentsBy>("orderBy", "createdAt");
const orderDirection = useRouteQuery<OrderDirection>("orderDirection", "desc");

const fetchParams = computed<ListIncidentsParams>(() => {
  let fromDate = timeRange.value ? {
    "-10m": new Date(Date.now() - 10 * 60 * 1000),
    "-1h": new Date(Date.now() - 3600 * 1000),
    "-6h": new Date(Date.now() - 6 * 3600 * 1000),
    "-12h": new Date(Date.now() - 12 * 3600 * 1000),
    "-24h": new Date(Date.now() - 24 * 3600 * 1000),
    "-7d": new Date(Date.now() - 7 * 24 * 3600 * 1000),
    "-30d": new Date(Date.now() - 30 * 24 * 3600 * 1000),
  }[timeRange.value] : null;

  return {
    pageNumber: pageNumber.value,
    status: includeStatuses.value,
    priority: null,
    itemsPerPage: 10,
    toDate: null,
    fromDate: fromDate ? fromDate.toISOString() : null,
    orderBy: orderBy.value,
    orderDirection: orderDirection.value
  }
});


const repository = useIncidentRepository();
const { data, refresh, status } = await repository.useIncidents(fetchParams);


const onClearFilters = () => {
  navigateTo({
    query: { pageNumber: pageNumber.value, statuses: allStatuses, timeRange: "null" },
  });
};

const hiddenIncidentsCount = computed(() => {
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
        $t("dashboard.mainSidebar.incidents")
        }}</BBreadcrumbItem>
    </BBreadcrumb>
    <h2>{{ $t("dashboard.incidents.pageTitle") }}</h2>
    <div class="small text-secondary mb-2">
      {{
        $t(
          "dashboard.incidents.totalIncidentCount",
          data?.totalNumberOfResults || 0
        )
      }}, {{ $t("dashboard.incidents.itemsPerPage", 10) }}
      <span v-if="hiddenIncidentsCount != 0">
        ,
        {{
          $t(
            "dashboard.incidents.filteredIncidentCount",
            hiddenIncidentsCount
          )
        }}
      </span>
    </div>
    <IncidentFilteringBar v-model:includeStatuses="includeStatuses" v-model:timeRange="timeRange"
      v-model:orderBy="orderBy" v-model:orderDirection="orderDirection" @clearFilters="onClearFilters" />
    <IncidentTableView v-if="data" :incidents="data!.items" />
    <div v-if="data?.totalNumberOfResults == 0" class="text-secondary text-center my-5">
      <Icon name="ph:seal-check-duotone" size="120px" />
      <h3>{{ $t("dashboard.incidents.emptyPage.title") }}</h3>
      <p class="lead">
        {{ $t("dashboard.incidents.emptyPage.text") }}
      </p>
    </div>
    <div v-else-if="data?.totalNumberOfFilteredResults == 0" class="text-secondary text-center my-5">
      <Icon name="ph:seal-check-duotone" size="120px" />
      <h3>{{ $t("dashboard.incidents.noResults.title") }}</h3>
      <p class="lead">
        {{ $t("dashboard.incidents.noResults.text") }}
      </p>
      <BButton variant="outline-secondary" @click="onClearFilters">{{ $t("dashboard.incidents.clearFilters") }}
      </BButton>
    </div>
    <BPagination v-model="pageNumber" v-if="data?.totalNumberOfFilteredResults! > 10" :prev-text="$t('pagination.prev')"
      pills :next-text="$t('pagination.next')" :total-rows="data?.totalNumberOfFilteredResults" :per-page="10" />
  </BContainer>
</template>
