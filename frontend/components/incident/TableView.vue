<script setup lang="ts">
import type { IncidentWithUsers } from 'bindings/IncidentWithUsers';

const localePath = useLocalePath();

const { incidents, showColumns = ["date", "acknowledgedBy", "status", "source", "rootCause"] } = defineProps<{
  incidents: IncidentWithUsers[],
  showColumns?: ("date" | "acknowledgedBy" | "status" | "source" | "rootCause")[]
}>() 
</script>

<template>
  <div class="incidents-table mb-3 mt-4">
    <div class="row head-row mb-2 d-none d-lg-flex text-muted">
      <div class="col" v-if="showColumns.includes('date')">
        <Icon name="ph:calendar-duotone" aria-hidden /> {{ $t('dashboard.incidents.date') }}
      </div>
      <div class="col" v-if="showColumns.includes('acknowledgedBy')">
        <Icon name="ph:users" aria-hidden />
        {{ $t('dashboard.incidents.people') }}
      </div>
      <div class="col" v-if="showColumns.includes('status')">
        <Icon name="ph:circle-dashed" aria-hidden />
        {{ $t('dashboard.incidents.status') }}
      </div>
      <div class="col" v-if="showColumns.includes('source')">
        <Icon name="ph:cylinder" aria-hidden />
        {{ $t('dashboard.incidents.source') }}
      </div>
      <div class="col" v-if="showColumns.includes('rootCause')">
        <Icon name="ph:siren" aria-hidden />
        {{ $t('dashboard.incidents.rootCause') }}
      </div>
    </div>
    <NuxtLink class="card mb-3 shadow-sm slide-up-fade-in" v-for="incident in incidents" :key="incident.id"
      :to="localePath(`/dashboard/incidents/${incident.id}`)">
      <div class="card-body">
        <div class="row">
          <!-- Date -->
          <div class="col-lg" v-if="showColumns.includes('date')">
            <label class="text-secondary d-flex align-items-center gap-1">
              <Icon name="ph:clock" aria-hidden /> {{ $t('dashboard.incidents.startedAt') }}
            </label>
            {{ $d(new Date(incident.createdAt), "long") }}
            <template v-if="incident.resolvedAt">
              <label class="text-secondary d-flex align-items-center gap-1">
                <Icon name="ph:check-circle" aria-hidden /> {{ $t('dashboard.incidents.resolvedAt') }}
              </label>
              {{ $d(new Date(incident.resolvedAt), "long") }}
            </template>
          </div>

          <!-- Acknowledged by -->
          <div class="col-lg" v-if="showColumns.includes('acknowledgedBy')">
            <label class="d-lg-none mt-2 text-secondary d-block">{{ $t('dashboard.incidents.people') }}</label>
            <div class="text-secondary" v-if="incident.acknowledgedBy.length === 0">
              --
            </div>
            <UserAvatar v-for="user in incident.acknowledgedBy" :key="user.id" :user="user" showTooltip />
          </div>

          <!-- Status -->
          <div class="col-lg" :class="{ 'text-danger': incident.status == 'ongoing' }"
            v-if="showColumns.includes('status')">
            <label class="d-lg-none mt-2 text-secondary d-block">{{ $t('dashboard.incidents.status') }}</label>
            {{ $t(`dashboard.incidentStatus.${incident.status}`) }}
          </div>

          <!-- Source -->
          <div class="col-lg" v-if="showColumns.includes('source')">
            <label class="d-lg-none mt-2 text-secondary d-block">{{ $t('dashboard.incidents.source') }}</label>
            <IncidentSource :incident-source-id="incident.incidentSourceId"
              :incident-source-type="incident.incidentSourceType" />
          </div>

          <!-- Root cause -->
          <div class="col-lg" v-if="showColumns.includes('rootCause')">
            <label class="d-lg-none mt-2 text-secondary d-block">{{ $t('dashboard.incidents.rootCause') }}</label>
            <IncidentCause :incident="incident" concise />
          </div>
        </div>
      </div>
    </NuxtLink>
  </div>
</template>

<style lang="scss" scoped>
.card {
  text-decoration: none;
}

@for $i from 1 through 10 {
  @keyframes slideUpFadeIn#{$i} {
    0% {
      opacity: 0;
      transform: translateY(30px);
    }

    #{$i* 10 + "%"} {
      opacity: 0;
      transform: translateY(30px);
    }

    100% {
      opacity: 1;
      transform: translateY(0);
    }
  }
}

@keyframes slideUpFadeIn {
  from {
    opacity: 0;
    transform: translateY(30px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.slide-up-fade-in {
  @for $i from 1 through 10 {
    &:nth-child(#{$i}n) {
      animation: slideUpFadeIn#{$i} 0.3s ease-out;
    }
  }

}
</style>