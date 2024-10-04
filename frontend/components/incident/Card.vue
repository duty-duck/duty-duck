<script setup lang="ts">
import type { Incident } from "bindings/Incident";
const localePath = useLocalePath();
const incident = defineProps<Incident>();
</script>

<template>
  <BCard class="mb-3" :class="{
    'border-danger': incident.status == 'ongoing',
  }">
    <dl>
      <dt>{{ $t('dashboard.incidents.status') }}</dt>
      <dd :class="{ 'text-danger': incident.status == 'ongoing' }">
        {{ $t(`dashboard.incidentStatus.${incident.status}`) }}
      </dd>
      <dt>{{ $t('dashboard.incidents.source') }}</dt>
      <dd>
        <IncidentCardHttpMonitorDetails v-if="incident.incidentSourceType == 'httpmonitor'"
          :http-monitor-id="incident.incidentSourceId" :incident="incident" />
      </dd>
      <dt>{{ $t('dashboard.incidents.rootCause') }}</dt>
      <dd>
        <IncidentCause :incident="incident" />
      </dd>
      <dt>
        <Icon name="ph:clock" aria-hidden /> {{ $t('dashboard.incidents.startedAt') }}
      </dt>
      <dd>{{ $d(new Date(incident.createdAt), "long") }}</dd>
      <dt>
        <Icon name="ph:check-circle" aria-hidden /> {{ $t('dashboard.incidents.resolvedAt') }}
      </dt>
      <dl v-if="incident.resolvedAt">
        {{ $d(new Date(incident.resolvedAt), "long") }}
      </dl>
      <dl v-else>--</dl>
    </dl>
  </BCard>
</template>

<style scoped lang="scss">
dl {
  dt {
    color: var(--bs-gray-800);
    font-weight: lighter;
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  dd {

    margin-bottom: 1rem;
  }
}
</style>
