<script lang="ts" setup>
import type { IncidentStatus } from 'bindings/IncidentStatus';
import type { TimeRange } from '../dashboard/TimeRangePicker.vue';
import type { OrderIncidentsBy } from 'bindings/OrderIncidentsBy';
import type { OrderDirection } from 'bindings/OrderDirection';

const { shownFilters = ["statuses", "timeRange", "orderBy"] } = defineProps<{
    shownFilters?: ("statuses" | "timeRange" | "orderBy")[];
}>();
const includeStatuses = defineModel<IncidentStatus[]>("includeStatuses", { required: true });
const timeRange = defineModel<TimeRange>("timeRange", { required: true });
const orderBy = defineModel<OrderIncidentsBy>("orderBy", { required: true });
const orderDirection = defineModel<OrderDirection>("orderDirection", { required: true });

const emit = defineEmits<{
    (e: 'clearFilters'): void;
}>();
</script>
<template>
    <nav class="filtering-bar d-flex gap-2 mb-4 py-3">
        <BButton class="flex-shrink-0 icon-link" variant="outline-secondary" @click="emit('clearFilters')">
            <Icon size="1.3rem" name="ph:funnel-simple-x-bold" />
        </BButton>
        <DashboardTimeRangePicker v-if="shownFilters.includes('timeRange')" v-model="timeRange" />
        <IncidentStatusDropdown v-if="shownFilters.includes('statuses')" v-model="includeStatuses" />
        <IncidentOrderByDropdown v-if="shownFilters.includes('orderBy')" v-model:orderBy="orderBy" v-model:orderDirection="orderDirection" />
    </nav>
</template>

<style lang="scss" scoped>
@import "~/assets/main.scss";

.filtering-bar {
  @include blurry-gray-background;
  display: flex;
  position: sticky;
  top: 50px;
  z-index: 1;
  flex-wrap: wrap;
}
</style>