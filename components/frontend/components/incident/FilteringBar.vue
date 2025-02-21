<script lang="ts" setup>
import type { IncidentStatus } from 'bindings/IncidentStatus';
import type { OrderIncidentsBy } from 'bindings/OrderIncidentsBy';
import type { OrderDirection } from 'bindings/OrderDirection';
import type { MetadataFilter } from 'bindings/MetadataFilter';

const { shownFilters = ["statuses", "timeRange", "orderBy"], metadataFilter } = defineProps<{
  shownFilters?: ("statuses" | "timeRange" | "orderBy" | "metadata")[];
  metadataFilter?: MetadataFilter
}>();
const includeStatuses = defineModel<IncidentStatus[]>("includeStatuses", { required: true });
const dateRange = defineModel<{ start: Date, end: Date } | null>("dateRange");
const orderBy = defineModel<OrderIncidentsBy>("orderBy", { required: true });
const orderDirection = defineModel<OrderDirection>("orderDirection", { required: true });

const metadataFilterCount = computed(() => {
  return Object.values(metadataFilter?.items ?? {}).filter(f => f?.length! > 0).length;
});

const emit = defineEmits<{
  clearFilters: [],
  toggleMetadata: [],
}>();
</script>
<template>
  <nav class="filtering-bar d-flex gap-2 mb-3">
    <BButton class="flex-shrink-0 icon-link" variant="outline-secondary" @click="emit('clearFilters')">
      <Icon size="1.3rem" name="ph:funnel-simple-x-bold" />
    </BButton>
    <DashboardTimeRangePicker v-if="shownFilters.includes('timeRange')" v-model="dateRange" />
    <IncidentStatusDropdown v-if="shownFilters.includes('statuses')" v-model="includeStatuses" />
    <BButton v-if="shownFilters.includes('metadata')" variant="outline-secondary"
      class="d-flex align-items-center gap-1" @click="emit('toggleMetadata')">
      <Icon name="ph:funnel" aria-hidden size="1.3rem" />
      {{ $t('dashboard.facets.title') }}
      <span v-if="metadataFilterCount">({{ metadataFilterCount }})</span>
    </BButton>
    <IncidentOrderByDropdown v-if="shownFilters.includes('orderBy')" v-model:orderBy="orderBy"
      v-model:orderDirection="orderDirection" />
  </nav>
</template>

<style lang="scss" scoped>
@import "~/assets/main.scss";

.filtering-bar {
  display: flex;
  align-items: center;
  position: sticky;
  top: $navbar-height;
  z-index: 20;
  flex-wrap: wrap;
  min-height: $navbar-height;

  @include media-breakpoint-down(lg) {
    @include blurry-gray-background;
  }

  @include media-breakpoint-up(lg) {
    top: 0px;
  }
}
</style>