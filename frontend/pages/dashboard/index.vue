<script setup lang="ts">
let httpMonitorRepo = useHttpMonitorRepository();
let incidentRepo = useIncidentRepository();
let { refresh: refreshIncidentCount, data: incidentCount } = await incidentRepo.useOngoingIncidentsCount();
let { refresh: refreshDownMonitorsCount, data: downMonitorsCount } = await httpMonitorRepo.useDownMonitorsCount();
</script>
<template>
  <main>
    <h2>Home</h2>
    <h3>Overview</h3>
    <div class="row row-gap-2">
      <div class="col-sm-6 col-md-3">
        <BCard class="text-center">
          <p class="h1 d-flex align-items-center justify-content-center">
            <Icon name="ph:warning-circle-duotone" v-if="incidentCount"/>
            <Icon name="ph:check-circle-duotone" v-else />
            {{ incidentCount }}
          </p>
          <h4 class="h6">Ongoing incidents</h4>
          <NuxtLink to="/dashboard/incidents" class="icon-link">
            See incidents
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
          <h4 class="h6">Monitors down or suspicious</h4>
          <NuxtLink to="/dashboard/monitors" class="icon-link">
            See Monitors
            <Icon name="ph:arrow-right"/>
          </NuxtLink>
        </BCard>
      </div>
    </div>

  </main>
</template>
