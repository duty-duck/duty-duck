<script lang="ts" setup>
import type { IncidentSourceType } from 'bindings/IncidentSourceType';

const { incidentSourceId, incidentSourceType } = defineProps<{ incidentSourceId: string, incidentSourceType: IncidentSourceType }>();
</script>

<template>
    <IncidentHttpMonitorSourceDetails :http-monitor-id="incidentSourceId" v-if="incidentSourceType == 'httpmonitor'" />
</template>
