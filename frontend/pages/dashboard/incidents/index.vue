<script lang="ts" setup>
import { useIntervalFn } from "@vueuse/core";
import type { IncidentStatus } from "bindings/IncidentStatus";
import type { ListIncidentsParams } from "bindings/ListIncidentsParams";
import type { TimeRange } from "~/components/dashboard/TimeRangePicker.vue";
import { allStatuses } from "~/components/incident/StatusDropdown.vue";

const localePath = useLocalePath();

const route = useRoute();

const pageNumber = computed({
  get() {
    return route.query.page ? Number(route.query.page) : 1
  },
  set(value: number) {
    navigateTo({
      query: { page: value, statuses: includeStatuses.value, timeRange: timeRange.value },
    });
  }
});

const timeRange = computed<TimeRange | null>({
  get() {
    // no filtering if the timeRange query param is explicitly set to null
    // keep only the last 7 days as default
    return route.query.timeRange == "null" ? null : (route.query.timeRange as TimeRange ?? "-7d");
  },
  set(value: TimeRange) {
    navigateTo({ query: { pageNumber: pageNumber.value, statuses: includeStatuses.value, timeRange: value ?? "null" } });
  }
});

const includeStatuses = computed<IncidentStatus[]>({
  get() {
    return route.query.statuses ? (route.query.statuses as IncidentStatus[]) : ["ongoing"];
  },
  set(value: IncidentStatus[]) {
    navigateTo({ query: { pageNumber: pageNumber.value, statuses: value, timeRange: timeRange.value } });
  }
});

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
    fromDate: fromDate ? fromDate.toISOString() : null
  }
});


const repository = await useIncidentRepository();
const { data, refresh } = await repository.useIncidents(fetchParams);


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
    <nav class="filtering-bar d-flex gap-2 mb-4 py-3">
      <BButton class="flex-shrink-0 icon-link" variant="outline-secondary" @click="onClearFilters">
        <Icon size="1.3rem" name="ph:funnel-simple-x-bold" />
      </BButton>
      <DashboardTimeRangePicker v-model="timeRange" />
      <IncidentStatusDropdown v-model="includeStatuses" />
    </nav>
    <div v-if="data?.totalNumberOfResults == 0" class="text-secondary text-center my-5">
      <Icon name="ph:seal-check-duotone" size="120px" />
      <h3>{{ $t("dashboard.incidents.emptyPage.title") }}</h3>
      <p class="lead">
        {{ $t("dashboard.incidents.emptyPage.text") }}
      </p>
    </div>
    <IncidentTableView v-if="data" :incidents="data!.items" />
    <BPagination
      v-model="pageNumber"
      v-if="data?.totalNumberOfFilteredResults! > 10"
      :prev-text="$t('pagination.prev')"
      pills
      :next-text="$t('pagination.next')"
      :total-rows="data?.totalNumberOfFilteredResults"
      :per-page="10"
    />
  </BContainer>
</template>

<style lang="scss" scoped>
@import "~/assets/main.scss";

.filtering-bar {
  @include blurry-gray-background;
  display: flex;
  position: sticky;
  top: 50px;
  z-index: 1;
}
</style>
