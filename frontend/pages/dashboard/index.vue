<script setup lang="ts">
let httpMonitorRepo = useHttpMonitorRepository();
let incidentRepo = useIncidentRepository();
let { refresh: refreshIncidentCount, data: incidentCount } = await incidentRepo.useOngoingIncidentsCount();
let { refresh: refreshDownMonitorsCount, data: downMonitorsCount } = await httpMonitorRepo.useDownMonitorsCount();
</script>
<template>
  <BContainer>
    <h2>{{ $t('dashboard.home.pageTitle') }}</h2>
    <h3>{{ $t('dashboard.home.overview') }}</h3>
    <div class="row row-gap-2">
      <div class="col-sm-6 col-md-3">
        <BCard class="text-center">
          <p class="h1 d-flex align-items-center justify-content-center">
            <Icon name="ph:warning-circle-duotone" v-if="incidentCount"/>
            <Icon name="ph:check-circle-duotone" v-else />
            {{ incidentCount }}
          </p>
          <h4 class="h6">{{ $t('dashboard.home.ongoingIncidents', incidentCount || 0) }}</h4>
          <NuxtLink to="/dashboard/incidents" class="icon-link">
            {{ $t('dashboard.home.goToIncidents') }}
            <Icon name="ph:arrow-right"/>
          </NuxtLink>
        </BCard>
      </div>
      <div class="col-sm-6 col-md-3">
        <BCard class="text-center">
          <p class="h1 d-flex align-items-center justify-content-center">
            <Icon name="ph:warning-circle-duotone" v-if="downMonitorsCount"/>
            <Icon name="ph:check-circle-duotone" v-else />
            {{ downMonitorsCount }}
          </p>
          <h4 class="h6">{{ $t('dashboard.home.downMonitors', downMonitorsCount || 0) }}</h4>
          <NuxtLink to="/dashboard/monitors" class="icon-link">
            {{ $t('dashboard.home.goToMonitors') }}
            <Icon name="ph:arrow-right"/>
          </NuxtLink>
        </BCard>
      </div>
    </div>
  </BContainer>
</template>
