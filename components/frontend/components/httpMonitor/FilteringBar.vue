<script lang="ts" setup>
import type { HttpMonitorStatus } from "bindings/HttpMonitorStatus";
import type { MetadataFilter } from "bindings/MetadataFilter";

const includeStatuses = defineModel<HttpMonitorStatus[]>("includeStatuses", { required: true });
const query = defineModel<string>("query", { required: true });

const { metadataFilter } = defineProps<{
  metadataFilter: MetadataFilter
}>();

const metadataFilterCount = computed(() => {
  return Object.values(metadataFilter?.items ?? {}).filter(f => f?.length! > 0).length;
})

const emit = defineEmits<{
  toggleMetadata: [],
  clearFilters: []
}>();
</script>

<template>
  <nav class="filtering-bar gap-2">
    <BButton variant="outline-secondary" @click="emit('clearFilters')" class="d-flex align-items-center"
      :v-b-tooltip.hover.top="`${$t('dashboard.monitors.clearFilters')}`">
      <Icon size="1.3rem" name="ph:funnel-simple-x-bold" />
    </BButton>
    <HttpMonitorStatusDropdown v-model="includeStatuses" />
    <BButton variant="outline-secondary" class="d-flex align-items-center gap-1" @click="emit('toggleMetadata')">
      <Icon name="ph:funnel" aria-hidden size="1.3rem" />
      {{ $t('dashboard.facets.title') }}
      <span v-if="metadataFilterCount">({{ metadataFilterCount }})</span>
    </BButton>
    <BInput class="border border-secondary bg-transparent" v-model="query"
      :placeholder="$t('dashboard.monitors.search')" style="width: 300px; flex-grow: 1;" />
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
  min-height: $navbar-height;

  @include media-breakpoint-down(lg) {
    @include blurry-gray-background;
  }

  @include media-breakpoint-up(lg) {
    top: 0px;
  }
}
</style>