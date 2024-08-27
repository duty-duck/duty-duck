<script setup lang="ts">
import type { Incident } from "bindings/Incident";
import type { IncidentWithSources } from "bindings/IncidentWithSources";
const localePath = useLocalePath();
const incident = defineProps<IncidentWithSources>();
</script>

<template>
  <BCard
    class="mb-3"
    :class="{
      'border-danger': incident.status == 'ongoing',
    }"
  >
    <div class="row">
      <dl class="col">
        <dt>{{ $t('dashboard.incidents.status') }}</dt>
        <dd :class="{ 'text-danger': incident.status == 'ongoing' }">
          {{ $t(`dashboard.incidentStatus.${incident.status}`) }}
        </dd>
      </dl>
      <dl class="col">
      <dt>{{ $t('dashboard.incidents.source') }}</dt>
      <dd>
        <div v-for="s in incident.sources" :key="s.id">
          <template
          
            v-if="s.type == 'HttpMonitor'"
          >

          </template>
          <NuxtLink
            :to="localePath(`/dashboard/monitors/${s.id}`)"
            class="icon-link"
            v-b-tooltip.hover.top
            :title="$t('dashboard.incidents.goToSource')"
          >
            <Icon name="ph:pulse-duotone" size="22px" />
            {{ $t('dashboard.monitors.httpMonitor')}}
          </NuxtLink>
        </div>
      </dd>
    </dl>
    </div>

    <div class="row">
      <dl class="col">
        <dt><Icon name="ph:clock" aria-hidden /> {{ $t('dashboard.incidents.startedAt') }}</dt>
        <dd>{{ $d(new Date(incident.createdAt), "long") }}</dd>
      </dl>
      <dl class="col">
        <dt><Icon name="ph:check-circle" aria-hidden /> {{ $t('dashboard.incidents.resolvedAt') }}</dt>
        <dl v-if="incident.resolvedAt">
          {{ $d(new Date(incident.resolvedAt), "long") }}
        </dl>
        <dl v-else>--</dl>
      </dl>
    </div>
  </BCard>
</template>

<style scoped lang="scss">
dl {
  dt {
    color: var(--bs-gray-800);
    font-weight: lighter;
    text-transform: uppercase;
    font-size: 0.8rem;
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
}
</style>
