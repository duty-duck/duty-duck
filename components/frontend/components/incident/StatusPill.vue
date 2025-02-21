<script lang="ts" setup>
import type { IncidentStatus } from 'bindings/IncidentStatus';
import type { BaseColorVariant } from 'bootstrap-vue-next';

const { status, size = 'lg' } = defineProps<{
    status: IncidentStatus,
    size?: 'sm' | 'lg'
}>();

const variants: Record<IncidentStatus, [keyof BaseColorVariant, string]> = {
    ongoing: ['danger', 'ph:warning-circle-fill'],
    resolved: ['success', 'ph:check-circle-fill']
}

const fontSize = computed(() => {
    return { 'lg': '1rem', 'sm': '.75rem' }[size]
});
const iconSize = computed(() => {
    return { 'lg': '1.5rem', 'sm': '1rem' }[size]
})

</script>

<template>
    <BBadge :variant="variants[status][0]" class="rounded-pill pill text-white" :style="{ 'font-size': fontSize }">
        <Icon :name="variants[status][1]" :size="iconSize" />
        {{ $t(`dashboard.incidentStatus.${status}`) }}
    </BBadge>
</template>

<style scoped lang="scss">
.pill {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0.75rem 0.25rem 0.25rem;
}
</style>