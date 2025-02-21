<script setup lang="ts">
const httpMonitorRepo = useHttpMonitorRepository();
const incidentRepo = useIncidentRepository();
const auth = await useAuth();
const localePath = useLocalePath();
const thisDevice = await useThisDevice();
const suggestedActions = await useSuggestedActions();
const { refresh: refreshIncidentCount, data: incidentCount } = await incidentRepo.useOngoingIncidentsCount();
const { refresh: refreshDownMonitorsCount, data: downMonitorsCount } = await httpMonitorRepo.useDownMonitorsCount();
</script>
<template>
  <BContainer>
    <!-- Greeting -->
    <section class="my-5">
      <h1 class="display-4">{{ $t('dashboard.home.greeting', { name: auth.userProfile?.user.firstName }) }}</h1>
      <p class="lead text-secondary">{{ $t('dashboard.home.greetingDescription') }}</p>
    </section>


    <!-- Overview section -->
    <section class="mb-5">
      <h3 class="mb-3">{{ $t('dashboard.home.overview') }}</h3>
      <div class="row row-gap-2">
        <div class="col-sm-6 col-md-3">
          <BCard class="text-center">
            <p class="h1 d-flex align-items-center justify-content-center">
              <Icon name="ph:warning-circle-duotone" v-if="incidentCount" />
              <Icon name="ph:check-circle-duotone" v-else />
              {{ incidentCount }}
            </p>
            <h4 class="h6">{{ $t('dashboard.home.ongoingIncidents', incidentCount || 0) }}</h4>
            <NuxtLink to="/dashboard/incidents" class="icon-link">
              {{ $t('dashboard.home.goToIncidents') }}
              <Icon name="ph:arrow-right" />
            </NuxtLink>
          </BCard>
        </div>
        <div class="col-sm-6 col-md-3">
          <BCard class="text-center">
            <p class="h1 d-flex align-items-center justify-content-center">
              <Icon name="ph:warning-circle-duotone" v-if="downMonitorsCount" />
              <Icon name="ph:check-circle-duotone" v-else />
              {{ downMonitorsCount }}
            </p>
            <h4 class="h6">{{ $t('dashboard.home.downMonitors', downMonitorsCount || 0) }}</h4>
            <NuxtLink to="/dashboard/httpMonitors" class="icon-link">
              {{ $t('dashboard.home.goToMonitors') }}
              <Icon name="ph:arrow-right" />
            </NuxtLink>
          </BCard>
        </div>
      </div>
    </section>

    <!-- Suggested actions-->
    <section class="mb-4" v-if="suggestedActions.length">
      <h3 class="mb-4 fs-4">{{ $t('dashboard.home.suggestedActions') }}</h3>
      <div class="row">
        <div class="col-md-6 col-lg-4" v-for="action in suggestedActions">
          <!-- Phone number verification -->
          <BAlert variant="info" class="mb-3" :model-value="true">
            <h5 v-if="action.title">{{ action.title }}</h5>
            <p v-for="p in action.description ?? []">
              {{ p }}
            </p>
            <BButton v-for="cta in action.cta ?? []" :to="cta.link" variant="outline-info" class="icon-link">
              <Icon v-if="cta.icon" :name="cta.icon" />
              {{ cta.label }}
            </BButton>
          </BAlert>
        </div>
      </div>
    </section>
  </BContainer>
</template>
