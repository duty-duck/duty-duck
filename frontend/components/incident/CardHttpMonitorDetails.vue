<script lang="ts" setup>
import type { Incident } from 'bindings/Incident';

const localePath = useLocalePath();
const props = defineProps<{
    httpMonitorId: string,
    incident: Incident
}>();

const repo = useHttpMonitorRepository();
const { data: monitorRes, status } = await repo.useHttpMonitor(props.httpMonitorId, { lazy: true });
</script>

<template>
    <div>
        <NuxtLink :to="localePath(`/dashboard/httpMonitors/${props.httpMonitorId}`)" class="icon-link" v-b-tooltip.hover.top
            :title="$t('dashboard.incidents.goToSource')">
            <Icon name="ph:pulse-duotone" size="22px" />
            {{ $t('dashboard.monitors.httpMonitor') }}
        </NuxtLink>
    </div>
    <div v-if="status == 'pending'">
    <BSpinner small label="Small spinner" />
    </div>
    <div v-else-if="status == 'success'">
        <span> {{ monitorRes?.monitor.url }} </span>
    </div>
</template>