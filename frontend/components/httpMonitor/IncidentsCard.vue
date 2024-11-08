<script setup lang="ts">
import { useRouteQuery } from "@vueuse/router";
import type { ListIncidentsParams } from "bindings/ListIncidentsParams";
import type { OrderDirection } from "bindings/OrderDirection";
import type { OrderIncidentsBy } from "bindings/OrderIncidentsBy";
import type { ReadHttpMonitorResponse } from "bindings/ReadHttpMonitorResponse";

const localePath = useLocalePath();
const repo = await useHttpMonitorRepository();
const incidentPageNumber = useRouteQuery("incidentsPageNumber", 1, { transform: Number });
const currentTab = useRouteQuery("incidentsCurrentTab", "ongoing" as "ongoing" | "history");
const orderBy = useRouteQuery<OrderIncidentsBy>("orderBy", "createdAt");
const orderDirection = useRouteQuery<OrderDirection>("orderDirection", "desc");
const timeRange = useTimeRangeQuery();

const incidentsParams = computed<ListIncidentsParams>(() => {
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
    itemsPerPage: 5,
    pageNumber: incidentPageNumber.value,
    status: ["resolved"],
    priority: null,
    fromDate: fromDate ? fromDate.toISOString() : null,
    toDate: null,
    orderBy: orderBy.value,
    orderDirection: orderDirection.value,
  }
});


const { monitorResponse } = defineProps<{
  monitorResponse: ReadHttpMonitorResponse;
}>();

const {
  data: incidents,
  refresh: refreshIncidents,
} = await repo.useHttpMonitorIncidents(
  monitorResponse.monitor.id,
  incidentsParams,
);


const currentTabIndex = computed({
  get() {
    if (!monitorResponse.ongoingIncident) {
      return 1;
    }
    return currentTab.value == "ongoing" ? 0 : 1;
  },
  set(tab) {
    currentTab.value = tab == 0 ? "ongoing" : "history";
  }
})

const onClearFilters = () => {
  navigateTo({
    query: { pageNumber: incidentPageNumber.value, timeRange: "null" },
  });
}

watch(
  () => currentTab.value,
  (tab) => {
    if (tab == "history") {
      incidentPageNumber.value = 1;
      onClearFilters();
      refreshIncidents();
    }
  },
  { immediate: true }
);

defineExpose({
  refreshIncidents,
});
</script>

<template>
  <BTabs v-model="currentTabIndex">
    <BTab :disabled="!monitorResponse.ongoingIncident" lazy>
      <template #title>
        <span class="d-flex align-items-center">
          <Icon aria-label="Incident started at" name="ph:seal-warning" size="1.3rem" class="me-1" />
          {{ $t("dashboard.monitors.ongoingIncident") }}
        </span>
      </template>
      <div v-if="monitorResponse.ongoingIncident" class="mt-3">
        <h5>
          {{ $t("dashboard.monitors.ongoingIncident") }}
        </h5>
        <NuxtLink :to="localePath(`/dashboard/incidents/${monitorResponse.ongoingIncident.id}`)" class="icon-link mb-3">
          <Icon aria-hidden name="ph:arrow-up-right" size="1.3rem" />
          {{ $t("dashboard.incidents.goToIncident") }}
        </NuxtLink>

        <h5>
          {{ $t("dashboard.incidents.startOfIncident") }}
        </h5>
        {{ $d(new Date(monitorResponse.ongoingIncident.createdAt), "long") }}
        <p>
        <h5>{{ $t("dashboard.incidents.rootCause") }}:</h5>
        <IncidentCause :incident="monitorResponse.ongoingIncident" />
        </p>
        <LazyIncidentTimeline :incident-id="monitorResponse.ongoingIncident.id" :show-comment-editor="false" />
      </div>
    </BTab>
    <BTab class="px-0 pb-0">
      <template #title>
        <span class="d-flex align-items-center">
          <Icon aria-label="Incident history" name="ph:clock-counter-clockwise" size="1.3rem" class="me-1" />
          {{ $t("dashboard.monitors.incidentHistory") }}
        </span>
      </template>
      <div class="mt-3">
        <h5>{{ $t("dashboard.monitors.incidentHistory") }}</h5>
        <IncidentFilteringBar :shown-filters="['timeRange', 'orderBy']" @clear-filters="onClearFilters"
          :include-statuses="['resolved']" v-model:time-range="timeRange" v-model:orderBy="orderBy"
          v-model:orderDirection="orderDirection" />
        <BCard v-if="incidents?.items.length == 0" class="text-center text-secondary py-5 mt-3">
          <h5>{{ $t("dashboard.monitors.noIncident.title") }} 👍</h5>
          <p>{{ $t("dashboard.monitors.noIncident.text") }}</p>
        </BCard>
        <div v-else class="mt-3">
          <IncidentTableView :incidents="incidents?.items!"
            :show-columns="['date', 'acknowledgedBy', 'status', 'rootCause']" />
          <BPagination v-model="incidentPageNumber" :prev-text="$t('pagination.prev')" pills limit="10"
            :next-text="$t('pagination.next')" :total-rows="incidents?.totalNumberOfFilteredResults || 0"
            :per-page="10" />
        </div>
      </div>
    </BTab>
  </BTabs>
</template>
