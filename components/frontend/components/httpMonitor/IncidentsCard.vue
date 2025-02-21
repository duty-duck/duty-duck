<script setup lang="ts">
import { useRouteQuery } from "@vueuse/router";
import type { ListIncidentsParams } from "bindings/ListIncidentsParams";
import type { OrderDirection } from "bindings/OrderDirection";
import type { OrderIncidentsBy } from "bindings/OrderIncidentsBy";
import type { ReadHttpMonitorResponse } from "bindings/ReadHttpMonitorResponse";

const localePath = useLocalePath();
const repo = useHttpMonitorRepository();
const incidentPageNumber = useRouteQuery("incidentsPageNumber", 1, { transform: Number });
const orderBy = useRouteQuery<OrderIncidentsBy>("orderBy", "createdAt");
const orderDirection = useRouteQuery<OrderDirection>("orderDirection", "desc");
const dateRange = useDateRangeQuery();

const incidentsParams = computed<ListIncidentsParams>(() => {
  return {
    itemsPerPage: 5,
    pageNumber: incidentPageNumber.value,
    status: ["resolved"],
    priority: null,
    fromDate: dateRange.value?.start.toISOString(),
    toDate: dateRange.value?.end.toISOString(),
    orderBy: orderBy.value,
    orderDirection: orderDirection.value,
    metadataFilter: null
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



const onClearFilters = () => {
  navigateTo({
    query: { pageNumber: incidentPageNumber.value, timeRange: "null" },
  });
}

defineExpose({
  refreshIncidents,
});
</script>

<template>
  <div>
    <!-- Ongoing Incident Section -->
    <section v-if="monitorResponse.ongoingIncident" class="mb-3">
      <div class="d-flex align-items-center mb-3">
        <Icon aria-label="Incident started at" name="ph:seal-warning-fill" size="1.3rem" class="me-2 text-danger" />
        <h5 class="mb-0">
          {{ $t("dashboard.monitors.ongoingIncident") }}
        </h5>
      </div>

      <BCard class="mb-3">
        <h6>{{ $t("dashboard.incidents.rootCause") }}:</h6>
        <IncidentCause :incident="monitorResponse.ongoingIncident" />
      </BCard>

      <NuxtLink :to="localePath(`/dashboard/incidents/${monitorResponse.ongoingIncident.id}`)" class="icon-link mb-3">
        <Icon aria-hidden name="ph:arrow-up-right" size="1.3rem" />
        {{ $t("dashboard.incidents.goToIncident") }}
      </NuxtLink>
    </section>

    <!-- Incident History Section -->
    <section>
      <div class="d-flex align-items-center ">
        <Icon aria-label="Incident history" name="ph:clock-counter-clockwise" size="1.3rem" class="me-2" />
        <h5 class="mb-0">{{ $t("dashboard.monitors.incidentHistory") }}</h5>
      </div>

      <IncidentFilteringBar :shown-filters="['timeRange', 'orderBy']" @clear-filters="onClearFilters"
        :include-statuses="['resolved']" v-model:date-range="dateRange" v-model:orderBy="orderBy"
        v-model:orderDirection="orderDirection" />

      <BCard v-if="incidents?.items.length == 0" class="text-center text-secondary py-5 mt-3">
        <h5>{{ $t("dashboard.monitors.noIncident.title") }} 👍</h5>
        <p>{{ $t("dashboard.monitors.noIncident.text") }}</p>
      </BCard>

      <div v-else>
        <IncidentTableView :incidents="incidents?.items!"
          :show-columns="['date', 'acknowledgedBy', 'status', 'rootCause']" />
        <BPagination v-model="incidentPageNumber" :prev-text="$t('pagination.prev')" :next-text="$t('pagination.next')"
          :total-rows="incidents?.totalNumberOfFilteredResults || 0" :per-page="10" pills limit="10" />
      </div>
    </section>
  </div>
</template>
