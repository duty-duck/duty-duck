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
    <BDropdown variant="outline-secondary" toggle-class="d-flex align-items-center gap-1">
        <template #button-content>
            <Icon name="ph:funnel" size="1.3rem" />
            {{ $t('dashboard.monitors.status') }}
            <span v-if="model!.length < allStatuses.length">({{ model?.length }})</span>
        </template>
        <BDropdownItem v-for="s in allStatuses">
            <input :id="`${s}-checkbox`" :key="s" type="checkbox" v-model="model" :value="s" />
            <label :for="`${s}-checkbox`" style="width: 100%; text-transform: capitalize;" class="ps-2">{{ $t(`dashboard.monitorStatus.${s}`) }}</label>
        </BDropdownItem>

    </BDropdown>
</template>