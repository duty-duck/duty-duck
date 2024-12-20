<script setup lang="ts">
const httpMonitorRepo = useHttpMonitorRepository();
const incidentRepo = useIncidentRepository();
const auth = await useAuth();
const localePath = useLocalePath();
const thisDevice = await useThisDevice();
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
    <section class="mb-4">
      <h3 class="mb-4 fs-4">{{ $t('dashboard.home.suggestedActions') }}</h3>
      <div class="row">
        <div class="col-md">
          <!-- Phone number verification -->
          <BAlert variant="info" class="mb-3"
            :model-value="auth.userProfile && (!auth.userProfile.user.phoneNumber || !auth.userProfile.user.phoneNumberVerified)">
            <h5>{{ $t('dashboard.home.phoneNumberVerificationRequired') }}</h5>
            <p>
              {{ $t('dashboard.home.phoneNumberVerificationRequiredDescription') }}
            </p>
            <BButton :to="localePath('/dashboard/myAccount')" variant="outline-info" class="icon-link">
              <Icon name="ph:phone-call-fill" />
              {{ $t('dashboard.home.verifyPhoneNumber') }}
            </BButton>
          </BAlert>
        </div>

        <div class="col-md">
          <!-- Push notifications -->
          <BAlert variant="info" class="mb-push-notifications-alert" :model-value="!thisDevice">
            <h5>{{ $t('dashboard.home.pushNotificationsRequired') }}</h5>
            <p>
              {{ $t('dashboard.home.pushNotificationsRequiredDescription') }}
            </p>
            <BButton :to="localePath('/dashboard/myAccount')" variant="outline-info" class="icon-link">
              <Icon name="ph:bell-duotone" />
              {{ $t('dashboard.home.configurePushNotifications') }}
            </BButton>
          </BAlert>
        </div>
      </div>
    </section>
  </BContainer>
</template>
