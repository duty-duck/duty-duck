<script lang="ts" setup>
import type { Incident } from 'bindings/Incident';

const { incident, concise = false } = defineProps<{ incident: Incident, concise?: boolean }>();
</script>

<template>
    <template v-if="incident.cause?.causeType == 'HttpMonitorIncidentCause'">
        <div v-if="!concise"> {{ $t("dashboard.httpMonitorIncidents.httpMonitorFailure") }}</div>
        <HttpMonitorIncidentLabel :incident="incident" />
    </template>
</template>