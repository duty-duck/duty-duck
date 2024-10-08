<script lang="ts" setup>
import { useRouteQuery } from '@vueuse/router'
import { useIntervalFn, useThrottleFn } from "@vueuse/core";
import type { ListIncidentsParams } from "bindings/ListIncidentsParams";
import humanizeDuration from "humanize-duration";

ensurePemissionOnBeforeMount("readHttpMonitors");

const localePath = useLocalePath();
const repo = useHttpMonitorRepository();
const route = useRoute();
const now = ref(new Date());
const toggleIsLoading = ref(false);
const { locale } = useI18n();

const incidentPageNumber = useRouteQuery("incidentsPageNumber", 1, { transform: Number });
const incidentsCardCurrentTab = useRouteQuery("incidentsCurrentTab",  "ongoing" as "ongoing" | "history");
const incidentsParams = computed<ListIncidentsParams>(() => ({
  itemsPerPage: 10,
  pageNumber: incidentPageNumber.value,
  status: ["resolved"],
  priority: null,
}));

const { refresh: refreshMonitorData, data: monitorData } =
  await repo.useHttpMonitor(route.params.monitorId as string);

const {
  data: incidentsData,
  refresh: refreshIncidents,
} = await repo.useHttpMonitorIncidents(
  route.params.monitorId as string,
  incidentsParams
);

const refreshIncidentsThrottled = useThrottleFn(refreshIncidents, 5000);

const lastStatusChange = computed(() => {
  if (!monitorData.value?.monitor.lastStatusChangeAt) {
    return null;
  }
  const duration =
    now.value.getTime() -
    new Date(monitorData.value.monitor.lastStatusChangeAt).getTime();
  return humanizeDuration(duration, {
    maxDecimalPoints: 0,
    language: locale.value,
    // only display seconds if the duration is less than a day
    units:
      duration >= 24 * 60 * 60000
        ? ["y", "mo", "d", "h", "m"]
        : ["y", "mo", "d", "h", "m", "s"],
  });
});
const lastCheckedAtDuration = computed(() => {
  if (!monitorData.value?.monitor.lastPingAt) {
    return null;
  }
  const duration =
    now.value.getTime() -
    new Date(monitorData.value.monitor.lastPingAt).getTime();

  return humanizeDuration(duration, {
    maxDecimalPoints: 0,
    language: locale.value,
    // only display seconds if the duration is less than a day
    units:
      duration >= 24 * 60 * 60000
        ? ["y", "mo", "d", "h", "m"]
        : ["y", "mo", "d", "h", "m", "s"],
  });
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
    toggleIsLoading.value = false;
  }
};

useIntervalFn(() => {
  now.value = new Date();
}, 1000);

useIntervalFn(() => {
  refreshMonitorData();
  refreshIncidentsThrottled();
}, 5000);

watch(
  () => incidentsCardCurrentTab.value,
  (tab) => {
    if (tab == "history") {
      incidentPageNumber.value = 1;
      refreshIncidentsThrottled();
    }
  },
  { immediate: true }
);
</script>

<template>
  <BContainer v-if="monitorData?.monitor">
    <BBreadcrumb>
      <BBreadcrumbItem :to="localePath('/dashboard')">{{
        $t("dashboard.mainSidebar.home")
      }}</BBreadcrumbItem>
      <BBreadcrumbItem :to="localePath('/dashboard/httpMonitors')">{{
        $t("dashboard.mainSidebar.monitors")
      }}</BBreadcrumbItem>
      <BBreadcrumbItem active>
        {{ $t("dashboard.monitors.details") }}
      </BBreadcrumbItem>
    </BBreadcrumb>
    <div class="d-flex align-items-center my-5 gap-3">
      <HttpMonitorStatusIcon :status="monitorData?.monitor.status" class="mx-5"
        :animated="monitorData.monitor.status != 'inactive'" big />
      <div>
        <h2 class="h4">
          {{ monitorData?.monitor.url }}
        </h2>
        <HttpMonitorStatusLabel :status="monitorData?.monitor.status" />
        &nbsp;
        <span v-show="monitorData?.monitor" class="small text-secondary">
          {{
            $t("dashboard.monitors.lastCheckedOn", {
              date: $d(new Date(monitorData.monitor.lastPingAt!), "long"),
            })
          }}</span>
      </div>
    </div>
    <div class="mb-4 d-flex gap-2">
      <BButton class="icon-link" variant="outline-secondary" @click="toggleMonitor" :disabled="toggleIsLoading">
        <template v-if="monitorData.monitor.status == 'inactive'">
          <Icon name="ph:play-fill" />
          {{ $t("dashboard.monitors.start") }}
        </template>
        <template v-else>
          <Icon name="ph:pause-fill" />
          {{ $t("dashboard.monitors.pause") }}
        </template>
      </BButton>
      <BButton class="icon-link" variant="outline-secondary"
        :to="localePath(`/dashboard/httpMonitors/${route.params.monitorId}/edit`)">
        <Icon name="ph:pencil" />
        {{ $t("dashboard.monitors.edit") }}
      </BButton>
    </div>
    <p class="mt-2 text-secondary" v-if="monitorData.monitor.status == 'inactive'">
      {{ $t("dashboard.monitors.pausedMonitorNotice") }}
    </p>
    <div class="row mb-4 row-gap-3">
      <div class="col-md-6 col-lg-4">
        <BCard class="h-100">
          <p>{{ $t("dashboard.monitors.lastStatusChange") }}</p>
          <p class="h4">
            {{
              lastStatusChange
                ? $t("dashboard.monitors.dateAgo", { date: lastStatusChange })
                : "--"
            }}
          </p>
        </BCard>
      </div>
      <div class="col-md-6 col-lg-4">
        <BCard class="h-100">
          <p>{{ $t("dashboard.monitors.lastCheck") }}</p>
          <p class="h4">
            {{
              lastCheckedAtDuration
                ? $t("dashboard.monitors.dateAgo", {
                  date: lastCheckedAtDuration,
                })
                : "--"
            }}
          </p>
        </BCard>
      </div>
    </div>
    <HttpMonitorIncidentsCard :monitor="monitorData.monitor" :on-going-incident="monitorData.ongoingIncident"
      :incidents="incidentsData ?? null" v-model:incidents-page-number="incidentPageNumber"
      v-model:current-tab="incidentsCardCurrentTab" />
  </BContainer>
</template>
