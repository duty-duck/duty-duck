<script setup lang="ts">
import type { IncidentWithSources } from "bindings/IncidentWithSources";
const localePath = useLocalePath();
const incident = defineProps<IncidentWithSources>();
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
          <div v-for="s in incident.sources" :key="s.id">
            <IncidentCardHttpMonitorDetails :http-monitor-id="s.id" :incident="incident" v-if="s.type == 'HttpMonitor'" />
          </div>
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
