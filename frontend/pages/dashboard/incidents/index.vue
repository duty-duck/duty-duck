<script lang="ts" setup>
import { useIntervalFn, useBreakpoints, breakpointsBootstrapV5 } from "@vueuse/core";
import type { IncidentStatus } from "bindings/IncidentStatus";
import type { ListIncidentsParams } from "bindings/ListIncidentsParams";
import { allStatuses } from "~/components/incident/StatusDropdown.vue";

const breakpoints = useBreakpoints(breakpointsBootstrapV5);
const lgOrLarger = breakpoints.greaterOrEqual("lg");

const localePath = useLocalePath();
const path = localePath("/dashboard/incidents");
const route = useRoute();
const router = useRouter();

const pageNumber = computed({
  get() {
    return route.query.page ? Number(route.query.page) : 1
  },
  set(value: number) {
    router.push({
      path,
      query: { page: value, statuses: includeStatuses.value },
    });
  }
})


const includeStatuses = computed<IncidentStatus[]>(() =>
  route.query.statuses && route.query.statuses.length
    ? (route.query.statuses as IncidentStatus[])
    : ["ongoing"]
);

const fetchParams = computed<ListIncidentsParams>(() => ({
  pageNumber: pageNumber.value,
  status: includeStatuses.value,
  priority: null,
  itemsPerPage: 10,
}));

const repository = useIncidentRepository();
const { status, data, refresh } = await repository.useIncidents(fetchParams);

const onIncludeStatusChange = (statuses: IncidentStatus[]) => {
  router.push({
    path,
    query: { pageNumber: pageNumber.value, statuses },
  });
};
const onClearFilters = () => {
  router.push({
    path,
    query: { pageNumber: pageNumber.value, statuses: allStatuses },
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
          $t("dashboard.sidebar.incidents")
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
    </BContainer>
    <nav
      class="filtering-bar flex-column flex-md-row gap-2 mb-4 py-3 container"
    >
      <IncidentStatusDropdown
        :model-value="includeStatuses"
        @update:model-value="onIncludeStatusChange"
      />
      <BButton
        class="flex-shrink-0 icon-link"
        variant="outline-secondary"
        @click="onClearFilters"
      >
        <Icon name="ph:x-square-fill" />
        {{ $t("dashboard.incidents.clearFilters") }}
      </BButton>
    </nav>
    <BContainer>
      <BAlert variant="danger" :model-value="status == 'error'">
        Failed to fetch HTTP incidents from the server. Please try again.
      </BAlert>
      <div
        v-if="data?.totalNumberOfResults == 0"
        class="text-secondary text-center my-5"
      >
        <Icon name="ph:seal-check-duotone" size="120px" />
        <h3>{{ $t("dashboard.incidents.emptyPage.title") }}</h3>
        <p class="lead">
          {{ $t("dashboard.incidents.emptyPage.text") }}
        </p>
      </div>
      <IncidentTableView v-if="data && lgOrLarger" :incidents="data!.items" />
      <IncidentCard
        v-if="!lgOrLarger"
        v-for="incident in data?.items"
        :key="incident.id"
        v-bind="incident"
      />
      <BPagination
        v-model="pageNumber"
        :prev-text="$t('pagination.prev')"
        :next-text="$t('pagination.next')"
        :total-rows="data?.totalNumberOfFilteredResults"
        :per-page="10"
      />
    </BContainer>
  </div>
</template>

<style lang="scss">
.filtering-bar {
  display: flex;
  position: sticky;
  top: 50px;
  z-index: 1;
  backdrop-filter: blur(10px);
  background-color: rgba(248, 249, 250, 0.6);
}
</style>
