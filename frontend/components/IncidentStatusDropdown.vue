<script lang="ts">
import type { IncidentStatus } from 'bindings/IncidentStatus';
export const allStatuses: IncidentStatus[] = [
    "resolved",
    "ongoing",
];
</script>

<script setup lang="ts">
const model = defineModel<IncidentStatus[]>();
</script>

<template>
    <BDropdown variant="outline-secondary">
        <template #button-content>
            <Icon name="ph:funnel-fill" />
            {{ $t('dashboard.incidents.filterByStatus') }}
            <span v-if="model!.length < allStatuses.length">({{ model?.length }})</span>
        </template>
        <BDropdownItem v-for="s in allStatuses">
            <input :id="`${s}-checkbox`" :key="s" type="checkbox" v-model="model" :value="s" />
            <label :for="`${s}-checkbox`" style="width: 100%; text-transform: capitalize;" class="ps-2">{{ $t(`dashboard.incidentStatus.${s}`) }}</label>
        </BDropdownItem>
    </BDropdown>
</template>