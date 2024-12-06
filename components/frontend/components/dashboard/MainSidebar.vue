<script setup lang="ts">
import { useIntervalFn } from '@vueuse/core';
const route = useRoute();
let incidentRepo = await useIncidentRepository();
const localePath = useLocalePath()
const { userHasPermissionComputed } = await useAuth();
const canReadHttpMonitors = userHasPermissionComputed('readHttpMonitors');
const canReadIncidents = userHasPermissionComputed('readIncidents');
const canReadTasks = userHasPermissionComputed('readTasks');

let { refresh: refreshIncidentCount, data: incidentCount } = await incidentRepo.useOngoingIncidentsCount();
useIntervalFn(() => refreshIncidentCount(), 20000);
watch(() => route.fullPath, () => refreshIncidentCount());
</script>

<template>
  <div class="py-2 px-lg-2">
    <ul class="nav nav-pills nav-light nav-fill flex-column gap-2">
      <li class="nav-item">
        <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard')" :class="{ 'active': route.path === localePath('/dashboard') }">
          <Icon name="ph:house-simple-duotone" size="20px" />
          {{ $t("dashboard.mainSidebar.home") }}
        </NuxtLink>
      </li>
      <li class="nav-item">
        <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard/httpMonitors')"
          :disabled="!canReadHttpMonitors" :class="{ 'active': route.path.startsWith(localePath('/dashboard/httpMonitors')) }">
          <Icon name="ph:globe-duotone" size="22px" />
          {{ $t("dashboard.mainSidebar.monitors") }}
        </NuxtLink>
      </li>
      <li class="nav-item">
        <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard/tasks')" :disabled="!canReadTasks"
          :class="{ 'active': route.path.startsWith(localePath('/dashboard/tasks')) }">
          <Icon name="ph:pulse-duotone" size="22px" />
          {{ $t('dashboard.mainSidebar.tasks') }}
        </NuxtLink>
      </li>
      <li class="nav-item" id="incidents-nav-item">
        <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard/incidents')" :disabled="!canReadIncidents" :class="{ 'active': route.path.startsWith(localePath('/dashboard/incidents')) }">
          <Icon name="ph:seal-warning-duotone" size="22px" />
          {{ $t("dashboard.mainSidebar.incidents") }}
          <BBadge class="ms-2" variant="danger" id="incidents-badge" v-if="incidentCount && incidentCount > 0">{{ incidentCount }}
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

<style scoped lang="scss">
@import "~/assets/main.scss";

#incidents-nav-item {
  position: relative;
}

#incidents-badge {
  position: absolute;
  right: -5px;
  top: -5px;

  @include media-breakpoint-up(xxl) {
    top: unset;
    right: 15px;
  }
}
</style>