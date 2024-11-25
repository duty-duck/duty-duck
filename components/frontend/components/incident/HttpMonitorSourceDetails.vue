<script lang="ts" setup>
const localePath = useLocalePath();
const { httpMonitorId } = defineProps<{
    httpMonitorId: string,
}>();

const repo = useHttpMonitorRepository();
const { data: monitorRes } = await repo.useHttpMonitor(httpMonitorId);
</script>

<template>
    <div class="details-container">
        <NuxtLink :to="localePath(`/dashboard/httpMonitors/${httpMonitorId}`)" class="icon-link" v-b-tooltip.hover.top
            :title="$t('dashboard.incidents.goToSource')">
            <Icon name="ph:pulse-duotone" size="22px" />
            {{ $t('dashboard.monitors.httpMonitor') }}
        </NuxtLink>
        <h2> {{ monitorRes?.monitor.url }} </h2>
    </div>
</template>

<style lang="scss" scoped>
@import "~/assets/main.scss";

.details-container {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;

    h2 {
        font-size: 1rem;
        font-weight: normal;
    }
}
</style>