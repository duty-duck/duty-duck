<script lang="ts" setup>
import type { HttpMonitorStatus } from "bindings/HttpMonitorStatus";

const {includeStatuses, query} = defineProps<{
  includeStatuses: HttpMonitorStatus[];
  query: string;
}>();

const emit = defineEmits<{
  (e: 'updateIncludeStatuses', statuses: HttpMonitorStatus[]): void;
  (e: 'updateQuery', event: Event): void;
  (e: 'clearFilters'): void;
}>();

const onIncludeStatusChange = (statuses: HttpMonitorStatus[]) => {
  emit('updateIncludeStatuses', statuses);
};

const onQueryChange = (event: Event) => {
  emit('updateQuery', event);
};

const onClearFilters = () => {
  emit('clearFilters');
};
</script>

<template>
  <nav class="filtering-bar flex-column flex-md-row gap-2 py-3 container">
    <HttpMonitorStatusDropdown
      :model-value="includeStatuses"
      @update:model-value="onIncludeStatusChange"
    />
    <BInput
      class="border border-secondary bg-transparent"
      :model-value="query"
      @input="onQueryChange"
      :placeholder="$t('dashboard.monitors.search')"
    />
    <BButton
      class="flex-shrink-0 icon-link"
      variant="outline-secondary"
      @click="onClearFilters"
    >
      <Icon name="ph:x-square-fill" />
      {{ $t("dashboard.monitors.clearFilters") }}
    </BButton>
  </nav>
</template>

<style lang="scss" scoped>
.filtering-bar {
  display: flex;
  position: sticky;
  top: 50px;
  z-index: 1;
  backdrop-filter: blur(10px);
  background-color: rgba(248, 249, 250, 0.6);
}
</style>