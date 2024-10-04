<script setup lang="ts">
import type { Incident } from 'bindings/Incident';

const localePath = useLocalePath();

const props = defineProps<{
    incidents: Incident[]
}>() 
</script>

<template>
    <div class="incidents-table mb-3">
        <div class="row head-row mb-2 d-none d-lg-flex">
            <div class="col">
                <Icon name="ph:calendar-duotone" aria-hidden /> {{ $t('dashboard.incidents.date') }}
            </div>
            <div class="col">
                <Icon name="ph:circle-dashed" aria-hidden />
                {{ $t('dashboard.incidents.status') }}
            </div>
            <div class="col">
                <Icon name="ph:cylinder" aria-hidden />
                {{ $t('dashboard.incidents.source') }}
            </div>
            <div class="col">
                <Icon name="ph:siren" aria-hidden />
                {{ $t('dashboard.incidents.rootCause') }}
            </div>
        </div>
        <BCard v-for="incident in props.incidents" :key="incident.id" class="mb-3 shadow-sm slide-up-fade-in">
            <div class="row">
                <div class="col-lg">
                    <label class="d-lg-none text-secondary d-block">{{ $t('dashboard.incidents.date') }}</label>
                    <Icon name="ph:clock" aria-hidden /> {{ $t('dashboard.incidents.startedAt') }} {{ $d(new
                        Date(incident.createdAt), "long") }}
                    <div v-if="incident.resolvedAt">

                        <Icon name="ph:check-circle" aria-hidden /> {{ $t('dashboard.incidents.resolvedAt') }}: {{
                            $d(new Date(incident.resolvedAt), "long") }}
                    </div>
                </div>
                <div class="col-lg" :class="{ 'text-danger': incident.status == 'ongoing' }">
                    <label class="d-lg-none mt-2 text-secondary d-block">{{ $t('dashboard.incidents.status') }}</label>
                    {{ $t(`dashboard.incidentStatus.${incident.status}`) }}
                </div>
                <div class="col-lg">
                    <label class="d-lg-none mt-2 text-secondary d-block">{{ $t('dashboard.incidents.source') }}</label>
                    <IncidentCardHttpMonitorDetails :http-monitor-id="incident.incidentSourceId" :incident="incident"
                        v-if="incident.incidentSourceType == 'httpmonitor'" />
                </div>
                <div class="col-lg">
                    <label class="d-lg-none mt-2 text-secondary d-block">{{ $t('dashboard.incidents.rootCause') }}</label>
                    <IncidentCause :incident="incident" concise />
                </div>
            </div>
        </BCard>

    </div>
</template>

<style lang="scss" scoped>

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