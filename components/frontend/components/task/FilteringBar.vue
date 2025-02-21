<script lang="ts" setup>
import type { MetadataFilter } from 'bindings/MetadataFilter';
import type { TaskStatus } from 'bindings/TaskStatus';

const { metadataFilter } = defineProps<{
  metadataFilter?: MetadataFilter
}>();

const includeStatuses = defineModel<TaskStatus[]>("includeStatuses", { required: true });
const query = defineModel<string>("query", { required: true });

const metadataFilterCount = computed(() => {
  return Object.values(metadataFilter?.items ?? {}).filter(f => f?.length! > 0).length;
});

const emit = defineEmits<{
  clearFilters: [],
  toggleMetadata: []
}>();
</script>
<template>
  <nav class="filtering-bar d-flex gap-2 mb-3">
    <BButton class="flex-shrink-0 icon-link" variant="outline-secondary" @click="emit('clearFilters')">
      <Icon size="1.3rem" name="ph:funnel-simple-x-bold" />
    </BButton>
    <TaskStatusDropdown v-model="includeStatuses" />
    <BButton variant="outline-secondary" class="d-flex align-items-center gap-1" @click="emit('toggleMetadata')">
      <Icon name="ph:funnel" aria-hidden size="1.3rem" />
      {{ $t('dashboard.facets.title') }}
      <span v-if="metadataFilterCount">({{ metadataFilterCount }})</span>
    </BButton>
    <BInput class="border border-secondary bg-transparent" v-model="query" :placeholder="$t('dashboard.tasks.search')"
      style="width: 300px; flex-grow: 1;" />
    <slot />
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