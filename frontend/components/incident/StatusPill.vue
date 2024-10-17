<script lang="ts" setup>
import type { IncidentStatus } from 'bindings/IncidentStatus';
import type { BaseColorVariant } from 'bootstrap-vue-next';

const { status } = defineProps<{
    status: IncidentStatus
}>();

const variants: Record<IncidentStatus, [keyof BaseColorVariant, string]> = {
    ongoing: ['danger', 'ph:warning-circle-fill'],
    resolved: ['success', 'ph:check-circle-fill']
}

</script>

<template>
    <BBadge :variant="variants[status][0]" class="rounded-pill pill">
        <Icon :name="variants[status][1]" size="1.5rem" />
        {{ $t(`dashboard.incidentStatus.${status}`) }}
    </BBadge>
</template>

<style scoped lang="scss">
.pill {
    font-size: 1rem;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0.75rem 0.25rem 0.25rem;
}
</style>