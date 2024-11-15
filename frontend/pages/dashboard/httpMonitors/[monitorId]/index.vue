<script lang="ts" setup>
import { useIntervalFn, useNow } from "@vueuse/core";
import { usePermissionGrant } from "~/composables/authComposables";
import type { LazyHttpMonitorIncidentsCard } from '#build/components';

await usePermissionGrant("readHttpMonitors");

const localePath = useLocalePath();
const route = useRoute();
const now = useNow();
const toggleIsLoading = ref(false);
const incidentsCard = ref<InstanceType<typeof LazyHttpMonitorIncidentsCard>>();
const { locale } = useI18n();

const { refresh: refreshMonitorResponse, data: monitorResponse } =
  await useHttpMonitor(route.params.monitorId as string);


const lastStatusChange = computed(() => {
  if (!monitorResponse.value?.monitor.lastStatusChangeAt) {
    return null;
  }
  const duration =
    now.value.getTime() -
    new Date(monitorResponse.value.monitor.lastStatusChangeAt).getTime();

  return formatDuration(duration, locale.value);
});
const lastCheckedAtDuration = computed(() => {
  if (!monitorResponse.value?.monitor.lastPingAt) {
    return null;
  }
  const duration =
    now.value.getTime() -
    new Date(monitorResponse.value.monitor.lastPingAt).getTime();

  return formatDuration(duration, locale.value);
});

const toggleMonitor = async () => {
  toggleIsLoading.value = true;
  try {
    await repo.toggleHttpMonitor(route.params.monitorId as string);
    await refreshMonitorResponse();
    await incidentsCard.value?.refreshIncidents();
  } catch (e) {
    console.error("Failed to toggle monitor:", e);
  } finally {
    toggleIsLoading.value = false;
  }
};

useIntervalFn(() => {
  refreshMonitorResponse();
  incidentsCard.value?.refreshIncidents();
}, 5000);
</script>

<template>
  <BContainer v-if="monitorResponse">
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
    <div class="d-flex flex-wrap align-items-center my-5 py-3 row-gap-5 column-gap-3">
      <HttpMonitorStatusIcon :status="monitorResponse.monitor.status" class="mx-auto mx-md-5"
        :animated="monitorResponse.monitor.status != 'inactive'" big />
      <div>
        <h2 class="h4 url">
          {{ monitorResponse.monitor.url }}
        </h2>
        <HttpMonitorStatusLabel :status="monitorResponse.monitor.status" />
        &nbsp;
        <span v-show="monitorResponse.monitor" class="small text-secondary">
          {{
            $t("dashboard.monitors.lastCheckedOn", {
              date: $d(new Date(monitorResponse.monitor.lastPingAt!), "long"),
            })
          }}</span>
      </div>
    </div>
    <div class="mb-3 d-flex gap-2">
      <BButton class="icon-link" variant="outline-secondary" @click="toggleMonitor" :disabled="toggleIsLoading">
        <template v-if="monitorResponse.monitor.status == 'inactive'">
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
    <p class="mt-2 text-secondary" v-if="monitorResponse.monitor.status == 'inactive'">
      {{ $t("dashboard.monitors.pausedMonitorNotice") }}
    </p>
    <div class="row mb-5 row-gap-3">
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
    <div class="mb-5">
      <h5>{{ $t("dashboard.monitors.metadata") }}</h5>
      <DashboardMetadataInput read-only v-model="monitorResponse.monitor.metadata" />
    </div>
    <Suspense>
      <template #fallback>
        <BSpinner />
      </template>
      <LazyHttpMonitorIncidentsCard :monitor-response="monitorResponse" ref="incidentsCard" />
    </Suspense>
  </BContainer>
</template>

<style scoped lang="scss">
.url {
  word-break: break-all;
}
</style>