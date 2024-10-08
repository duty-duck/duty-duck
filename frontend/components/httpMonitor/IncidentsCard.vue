<script setup lang="ts">
import type { HttpMonitor } from "bindings/HttpMonitor";
import type { Incident } from "bindings/Incident";
import type { ListIncidentsResponse } from "bindings/ListIncidentsResponse";

const { onGoingIncident, incidents } = defineProps<{
  monitor: HttpMonitor;
  onGoingIncident: Incident | null;
  incidents: ListIncidentsResponse | null
}>();
const currentTab = defineModel<"ongoing" | "history">("currentTab", { required: true });
const currentTabIndex = computed({
  get() {
    if (!onGoingIncident) {
      return 1;
    }
    return currentTab.value == "ongoing" ? 0 : 1;
  },
  set(tab) {
    currentTab.value = tab == 0 ? "ongoing" : "history";
  }
})
const incidentsPageNumber = defineModel<number>("incidentsPageNumber", { required: true });
</script>

<template>
  <BTabs v-model="currentTabIndex" pills>
    <BTab :disabled="!onGoingIncident" lazy>
      <template #title>
        <span class="d-flex align-items-center">
          <Icon aria-label="Incident started at" name="ph:seal-warning" size="1.3rem" class="me-1" />
          {{ $t("dashboard.monitors.ongoingIncident") }}
        </span>
      </template>
      <BCard v-if="onGoingIncident" class="mt-3">
        <p>
          <span class="text-secondary">
            {{ $t("dashboard.incidents.startOfIncident") }}
          </span>
          <br />
          {{ $d(new Date(onGoingIncident.createdAt), "long") }}
        </p>
        <p>
          <span class="text-secondary">{{ $t("dashboard.incidents.rootCause") }}:</span><br />
          <IncidentCause :incident="onGoingIncident" />
        </p>
      </BCard>
    </BTab>
    <BTab class="px-0 pb-0">
      <template #title>
        <span class="d-flex align-items-center">
          <Icon aria-label="Incident history" name="ph:clock-counter-clockwise" size="1.3rem" class="me-1"  />
          {{ $t("dashboard.monitors.incidentHistory") }}
        </span>
      </template>
      <BCard class="mt-3" no-body >
        <BListGroup flush class="mb-3">
          <BListGroupItem href="#" v-for="i in incidents?.items" :key="i.id">
            <span class="icon-link">
              <Icon aria-label="Incident started at" name="ph:clock" />
              {{ $d(new Date(i.createdAt), "long") }}
            </span>
            <p class="fw-semibold">
              <HttpMonitorIncidentLabel :incident="i" />
            </p>
          </BListGroupItem>
        </BListGroup>
        <div class="px-3">
          <BPagination v-model="incidentsPageNumber" :prev-text="$t('pagination.prev')"
            :next-text="$t('pagination.next')" :total-rows="incidents?.totalNumberOfFilteredResults || 0"
            :per-page="10" />
        </div>
      </BCard>
    </BTab>
  </BTabs>
</template>