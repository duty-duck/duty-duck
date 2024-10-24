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
  <nav class="filtering-bar d-flex gap-2 py-3">
    <BButton
      variant="outline-secondary"
      @click="onClearFilters"
      class="d-flex align-items-center"
    >
      <Icon size="1.3rem" name="ph:funnel-simple-x-bold" />
    </BButton>
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
}
</style>