<script lang="ts" setup>
import type { HttpMonitorStatus } from "bindings/HttpMonitorStatus";

const includeStatuses = defineModel<HttpMonitorStatus[]>("includeStatuses", { required: true });
const query = defineModel<string>("query", { required: true });

const emit = defineEmits<{
  (e: 'clearFilters'): void;
}>();
</script>

<template>
  <nav class="filtering-bar d-flex gap-2 py-3">
    <BButton variant="outline-secondary" @click="emit('clearFilters')" class="d-flex align-items-center"
      :v-b-tooltip.hover.top="`${$t('dashboard.monitors.clearFilters')}`">
      <Icon size="1.3rem" name="ph:funnel-simple-x-bold" />
    </BButton>
    <HttpMonitorStatusDropdown v-model="includeStatuses" />
    <BInput
      class="border border-secondary bg-transparent"
      v-model="query"
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