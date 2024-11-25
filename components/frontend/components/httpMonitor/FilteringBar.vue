<script lang="ts" setup>
import type { HttpMonitorStatus } from "bindings/HttpMonitorStatus";

const includeStatuses = defineModel<HttpMonitorStatus[]>("includeStatuses", { required: true });
const query = defineModel<string>("query", { required: true });

const emit = defineEmits<{
  (e: 'clearFilters'): void;
}>();
</script>

<template>
  <nav class="filtering-bar gap-2">
    <BButton variant="outline-secondary" @click="emit('clearFilters')" class="d-flex align-items-center"
      :v-b-tooltip.hover.top="`${$t('dashboard.monitors.clearFilters')}`">
      <Icon size="1.3rem" name="ph:funnel-simple-x-bold" />
    </BButton>
    <HttpMonitorStatusDropdown v-model="includeStatuses" />
    <slot />
    <BInput
      class="border border-secondary bg-transparent"
      v-model="query"
      :placeholder="$t('dashboard.monitors.search')"
      style="width: 300px;"
    />
  </nav>
</template>

<style lang="scss" scoped>
@import "~/assets/main.scss";

.filtering-bar {
  display: flex;
  align-items: center;
  position: sticky;
  top: 50px;
  z-index: 10;
  flex-wrap: wrap;
  height: $navbar-height;

  @include media-breakpoint-down(lg) {
    @include blurry-gray-background;
  }

  @include media-breakpoint-up(lg) {
    top: 0px;
  }
}
</style>