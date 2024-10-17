<script setup lang="ts">
import { useIntervalFn } from '@vueuse/core';
const route = useRoute();
let incidentRepo = useIncidentRepository();
const localePath = useLocalePath()
const { canComputed } = useAuth();
const canReadHttpMonitors = canComputed('readHttpMonitors');
const canReadIncidents = canComputed('readIncidents');

let { refresh: refreshIncidentCount, data: incidentCount } = await incidentRepo.useOngoingIncidentsCount();
useIntervalFn(() => refreshIncidentCount(), 20000);
watch(() => route.fullPath, () => refreshIncidentCount());
</script>

<template>
  <div class="py-2 ps-lg-4 pe-lg-2 mt-lg-4">
    <ul class="nav nav-pills nav-light nav-fill flex-column gap-2">
      <li class="nav-item">
        <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard')" :class="{ 'active': route.fullPath === localePath('/dashboard') }">
          <Icon name="ph:house-simple-duotone" size="20px" />
          {{ $t("dashboard.mainSidebar.home") }}
        </NuxtLink>
      </li>
      <li class="nav-item">
        <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard/httpMonitors')"
          :disabled="!canReadHttpMonitors" :class="{ 'active': route.fullPath.startsWith(localePath('/dashboard/httpMonitors')) }">
          <Icon name="ph:pulse-duotone" size="22px" />
          {{ $t("dashboard.mainSidebar.monitors") }}
        </NuxtLink>
      </li>
      <li class="nav-item">
        <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard/incidents')" :disabled="!canReadIncidents" :class="{ 'active': route.fullPath.startsWith(localePath('/dashboard/incidents')) }">
          <Icon name="ph:seal-warning-duotone" size="22px" />
          {{ $t("dashboard.mainSidebar.incidents") }}
          <BBadge class="ms-2" variant="danger" v-if="incidentCount && incidentCount > 0">{{ incidentCount }}
          </BBadge>
        </NuxtLink>
      </li>
      <li class="nav-item">
        <a class="nav-link icon-link disabled" href="#">
          <Icon name="ph:speedometer-duotone" size="22px" />
          Web perf.
          <BBadge>{{ $t('dashboard.mainSidebar.soon') }}</BBadge>
        </a>
      </li>
      <li class="nav-item">
        <a class="nav-link icon-link disabled" href="#">
          <Icon name="ph:cpu-duotone" size="22px" />
          Infrastructure
          <BBadge>{{ $t('dashboard.mainSidebar.soon') }}</BBadge>
        </a>
      </li>
    </ul>
  </div>
</template>