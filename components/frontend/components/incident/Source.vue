<script lang="ts" setup>
import type { IncidentSourceType } from 'bindings/IncidentSourceType';
import ScheduledTaskSourceDetails from './ScheduledTaskSourceDetails.vue';

const { incidentSourceId, incidentSourceType } = defineProps<{ incidentSourceId: string, incidentSourceType: IncidentSourceType }>();
</script>

<template>
    <IncidentHttpMonitorSourceDetails :http-monitor-id="incidentSourceId" v-if="incidentSourceType == 'httpmonitor'" />
    <ScheduledTaskSourceDetails :task-id="incidentSourceId" v-if="incidentSourceType == 'task'" />
</template>
