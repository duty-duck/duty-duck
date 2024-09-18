<script lang="ts">
import type { HttpMonitorStatus } from 'bindings/HttpMonitorStatus';
export const allStatuses: HttpMonitorStatus[] = [
    "up",
    "down",
    "suspicious",
    "recovering",
    "inactive",
    "unknown"
];
</script>

<script setup lang="ts">
const model = defineModel<HttpMonitorStatus[]>();
</script>

<template>
    <BDropdown variant="outline-secondary">
        <template #button-content>
            <Icon name="ph:funnel-fill" />
            {{ $t('dashboard.monitors.filterByStatus') }}
            <span v-if="model!.length < allStatuses.length">({{ model?.length }})</span>
        </template>
        <BDropdownItem v-for="s in allStatuses">
            <input :id="`${s}-checkbox`" :key="s" type="checkbox" v-model="model" :value="s" />
            <label :for="`${s}-checkbox`" style="width: 100%; text-transform: capitalize;" class="ps-2">{{ $t(`dashboard.monitorStatus.${s}`) }}</label>
        </BDropdownItem>

    </BDropdown>
</template>