<script lang="ts" setup>
import { useIntervalFn, useThrottleFn } from "@vueuse/core";
import type { ListIncidentsParams } from "bindings/ListIncidentsParams";
import humanizeDuration from "humanize-duration";

const repo = useHttpMonitorRepository();
const route = useRoute();
const now = ref(new Date());
const toggleIsLoading = ref(false);

const incidentPageNumber = ref(1);
const incidentsParams = computed<ListIncidentsParams>(() => ({
  itemsPerPage: 15,
  pageNumber: incidentPageNumber.value,
  status: ['resolved'],
  priority: null
}));

const { refresh: refreshMonitorData, data: monitorData } = await repo.useHttpMonitor(route.params.monitorId as string);
const { data: incidentsData, refresh: refreshIncidents, status: incidentsStatus } = await repo.useHttpMonitorIncidents(route.params.monitorId as string, incidentsParams);

const incidentsCardCurrentTab = ref<"ongoing" | "history">(monitorData.value?.ongoingIncident ? "ongoing" : "history");
const refreshIncidentsThrottled = useThrottleFn(refreshIncidents, 2000);

const lastCheckedAt = computed(() => {
  if (!monitorData.value?.monitor.lastPingAt) {
    return null
  }
  const date = new Date(monitorData.value.monitor.lastPingAt);
  return date.toLocaleString("en-UK")
});
const lastStatusChange = computed(() => {
  if (!monitorData.value?.monitor.lastStatusChangeAt) {
    return null
  }
  return humanizeDuration(now.value.getTime() - new Date(monitorData.value.monitor.lastStatusChangeAt).getTime(), { maxDecimalPoints: 0 })
});
const lastCheckedAtDuration = computed(() => {
  if (!monitorData.value?.monitor.lastPingAt) {
    return null
  }
  return humanizeDuration(now.value.getTime() - new Date(monitorData.value.monitor.lastPingAt).getTime(), { maxDecimalPoints: 0 })
});

const toggleMonitor = async () => {
  toggleIsLoading.value = true;
  try {
    await repo.toggleHttpMonitor(route.params.monitorId as string);
    await refreshMonitorData();
    await refreshIncidents();
  } catch (e) {
    console.error("Failed to toggle monitor:", e);
  } finally {
    toggleIsLoading.value = false
  }
}

useIntervalFn(() => {
  now.value = new Date();
}, 1000);

useIntervalFn(() => {
  refreshMonitorData();
  refreshIncidentsThrottled();
}, 5000);

watch(() => incidentsCardCurrentTab.value, (tab) => {
  if (tab == 'history') {
    refreshIncidentsThrottled()
  }
}, { immediate: true })
</script>

<template>
  <BContainer v-if="monitorData?.monitor">
    <BBreadcrumb>
      <BBreadcrumbItem to="/dashboard">Home</BBreadcrumbItem>
      <BBreadcrumbItem to="/dashboard/monitors">Monitors</BBreadcrumbItem>
      <BBreadcrumbItem active>
        Details
      </BBreadcrumbItem>
    </BBreadcrumb>
    <div class="d-flex align-items-center my-5">
      <MonitorStatusIcon :status="monitorData?.monitor.status" class="mx-5"
        :animated="monitorData.monitor.status != 'inactive'" />
      <div>
        <h2 class="h4">
          {{ monitorData?.monitor.url }}
        </h2>
        <MonitorStatusLabel :status="monitorData?.monitor.status" />
        <span v-show="lastCheckedAt" class="small text-secondary"> &nbsp;Last checked on {{ lastCheckedAt }}</span>
      </div>
    </div>
    <div class="mb-4">
      <BButton class="icon-link" variant="outline-secondary" @click="toggleMonitor" :disabled="toggleIsLoading">
        <template v-if="monitorData.monitor.status == 'inactive'">
          <Icon name="ph:play-fill" />
          Start
        </template>
        <template v-else>
          <Icon name="ph:pause-fill" />
          Pause
        </template>
      </BButton>
      <p class="mt-2 text-secondary" v-if="monitorData.monitor.status == 'inactive'">
        This monitor is currently inactive. Click the start button to resume monitoring this endpoint.
      </p>
    </div>
    <div class="row mb-4 row-gap-3">
      <div class="col-md-6 col-lg-4">
        <BCard class="h-100">
          <p>Last status change</p>
          <p class="h4">{{ lastStatusChange || "--" }}</p>
        </BCard>
      </div>
      <div class="col-md-6 col-lg-4">
        <BCard class="h-100">
          <p>Last checked</p>
          <p class="h4">{{ lastCheckedAtDuration ? `${lastCheckedAtDuration} ago` : '--' }}</p>
        </BCard>
      </div>
    </div>
    <MonitorIncidentsCard :monitor="monitorData.monitor" :on-going-incident="monitorData.ongoingIncident"
      :incidents-page-number="incidentPageNumber" :incidents="{ data: incidentsData, status: incidentsStatus }"
      :current-tab="incidentsCardCurrentTab" @change-page="page => incidentPageNumber = page"
      @change-tab="tab => incidentsCardCurrentTab = tab" />
  </BContainer>
</template>