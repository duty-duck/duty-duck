<script lang="ts" setup>
import type { Incident } from 'bindings/Incident';
import type { IncidentWithUsers } from 'bindings/IncidentWithUsers';
import { getIncidentLabel } from '../httpMonitor/IncidentLabel.vue';

const { incident, concise = false } = defineProps<{ incident: Incident | IncidentWithUsers, concise?: boolean }>();
const { t } = useI18n();
const otherCausesTooltip = computed(() => {
    return incident.cause?.previousPings.map((ping) => `- ${getIncidentLabel(ping, t)}`).join('\n');
})
</script>

<template>
    <template v-if="incident.cause?.causeType == 'HttpMonitorIncidentCause'">
        <div v-if="!concise"> {{ $t("dashboard.httpMonitorIncidents.httpMonitorFailure") }}</div>
        <HttpMonitorIncidentLabel v-if="incident.cause?.lastPing" :cause="incident.cause.lastPing" />
        <div v-if="!concise && incident.cause?.previousPings.length > 0" class="text-secondary fw-normal mt-1">
            <span class="icon-link" v-b-tooltip.hover.top="otherCausesTooltip" style="cursor: help;">
                <Icon name="ph:plus" />
                {{ $t("dashboard.httpMonitorIncidents.otherCauses", { count: incident.cause.previousPings.length }) }}
            </span>
        </div>
    </template>
</template>