<script setup lang="ts">
const route = useRoute();
let incidentRepo = useIncidentRepository();
const localePath = useLocalePath()
const { userHasPermissionComputed } = await useAuth();
const canReadHttpMonitors = userHasPermissionComputed('readHttpMonitors');
const canReadIncidents = userHasPermissionComputed('readIncidents');
const canReadTasks = userHasPermissionComputed('readTasks');
const documentationPath = await useDocumentationFirstPage();

let { refresh: refreshIncidentCount, data: incidentCount } = await incidentRepo.useOngoingIncidentsCount();

// refresh incident count regularly
useDataRefreshInterval(refreshIncidentCount);

// refresh incident count when navigating to the incidents page
watch(() => route.path, (newPath, oldPath) => {
  if (!oldPath.includes('/incidents') && newPath.includes('/incidents')) {
    refreshIncidentCount();
  }
});
</script>

<template>
  <div class="py-2 px-lg-2 d-flex flex-column justify-content-between">
    <ul class="nav nav-pills nav-light nav-fill flex-column gap-2">
      <li class="nav-item">
        <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard')"
          :class="{ 'active': route.path === localePath('/dashboard') }">
          <Icon name="ph:house-simple-duotone" size="20px" />
          {{ $t("dashboard.mainSidebar.home") }}
        </NuxtLink>
      </li>
      <li class="nav-item">
        <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard/httpMonitors')" v-if="canReadHttpMonitors"
          :class="{ 'active': route.path.startsWith(localePath('/dashboard/httpMonitors')) }">
          <Icon name="ph:globe-duotone" size="22px" />
          {{ $t("dashboard.mainSidebar.monitors") }}
        </NuxtLink>
      </li>
      <li class="nav-item">
        <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard/tasks')" v-if="canReadTasks"
          :class="{ 'active': route.path.startsWith(localePath('/dashboard/tasks')) }">
          <Icon name="ph:pulse-duotone" size="22px" />
          {{ $t('dashboard.mainSidebar.tasks') }}
        </NuxtLink>
      </li>
      <li class="nav-item" id="incidents-nav-item">
        <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard/incidents')" v-if="canReadIncidents"
          :class="{ 'active': route.path.startsWith(localePath('/dashboard/incidents')) }">
          <Icon name="ph:seal-warning-duotone" size="22px" />
          {{ $t("dashboard.mainSidebar.incidents") }}
          <BBadge class="ms-2" variant="danger" id="incidents-badge" v-if="incidentCount && incidentCount > 0">{{
            incidentCount }}
          </BBadge>
        </NuxtLink>
      </li>
    </ul>
    <ul class="nav nav-pills nav-light nav-fill">
      <li class="nav-item">
        <NuxtLink target="_blank" class="nav-link icon-link" :to="documentationPath">
          <Icon name="ph:book-duotone" size="22px" />
          {{ $t('dashboard.mainSidebar.docs') }}
        </NuxtLink>
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