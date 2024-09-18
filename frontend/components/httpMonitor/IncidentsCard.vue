<script setup lang="ts">
import type { AsyncDataRequestStatus } from "#app";
import type { HttpMonitor } from "bindings/HttpMonitor";
import type { IncidentWithSources } from "bindings/IncidentWithSources";
import type { ListIncidentsResponse } from "bindings/ListIncidentsResponse";

const { monitor, onGoingIncident, incidents, currentTab } = defineProps<{
  monitor: HttpMonitor;
  onGoingIncident: IncidentWithSources | null;
  incidents: {
    status: AsyncDataRequestStatus;
    data: ListIncidentsResponse | null;
  };
  currentTab: "ongoing" | "history";
  incidentsPageNumber: number;
}>();
const emits = defineEmits<{
  changePage: [page: number];
  changeTab: [tab: "ongoing" | "history"];
}>();
</script>

<template>
  <BCard header-tag="nav" no-body>
    <template #header>
      <BNav card-header tabs>
        <BNavItem @click="emits('changeTab', 'ongoing')" :active="currentTab == 'ongoing' && !!onGoingIncident"
          :disabled="!onGoingIncident">
          <Icon v-if="onGoingIncident" name="ph:seal-warning-duotone" class="text-danger" size="1.2rem"
            style="position: relative; top: 0.2rem" />
          {{ $t("dashboard.monitors.ongoingIncident") }}
        </BNavItem>
        <BNavItem @click="emits('changeTab', 'history')" :active="currentTab == 'history' || !onGoingIncident">
          {{ $t("dashboard.monitors.incidentHistory") }}
        </BNavItem>
      </BNav>
    </template>
    <BCardBody v-if="currentTab == 'ongoing' && onGoingIncident">
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
    </BCardBody>
    <template v-else-if="incidents.data?.items.length == 0">
      <BCardBody class="text-center py-5">
        <h5>{{ $t("dashboard.monitors.noIncident.title") }}</h5>
        <p>{{ $t("dashboard.monitors.noIncident.text") }}</p>
      </BCardBody>
    </template>
    <template v-else-if="incidents.data">
      <BListGroup flush>
        <BListGroupItem href="#" v-for="i in incidents?.data?.items" :key="i.id">
          <span class="icon-link">
            <Icon aria-label="Incident started at" name="ph:clock" />
            {{ $d(new Date(i.createdAt), "long") }}
          </span>
          <p class="fw-semibold">
            <HttpMonitorIncidentLabel :incident="i" />
          </p>
        </BListGroupItem>
      </BListGroup>

      <BCardBody>
        <BPagination :modelValue="incidentsPageNumber" @update:modelValue="(page: number) => emits('changePage', page)"
          :prev-text="$t('pagination.prev')" :next-text="$t('pagination.next')"
          :total-rows="incidents!.data!.totalNumberOfFilteredResults" :per-page="10" />
      </BCardBody>
    </template>
  </BCard>
</template>
