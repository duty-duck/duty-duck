<script setup lang="ts">
import type { HttpMonitor } from "bindings/HttpMonitor";

const monitor = defineProps<HttpMonitor>();
const date = computed(() => {
  if (!monitor.lastPingAt) {
    return null
  }
  const date = new Date(monitor.lastPingAt);
  return date.toLocaleString("en-UK")
})
</script>

<template>
  <NuxtLink :to="`/dashboard/monitors/${monitor.id}`" class="card mb-3 shadow-sm" :class="{
    'border-danger': monitor.status == 'down',
    'border-warning': monitor.status == 'suspicious',
    'border-info': monitor.status == 'recovering',
    'border-secondary':
      monitor.status == 'unknown' || monitor.status == 'inactive',
    'border-success': monitor.status == 'up',
  }">
    <BCardBody class="d-flex align-items-center px-2">
      <MonitorStatusIcon :status="monitor.status" class="mx-4 mx-lg-5" />
      <div class="flex-grow-1">
        <div class="h6">
          {{ monitor.url }}
        </div>
        <MonitorStatusLabel :status="monitor.status" />
        <span v-show="date" class="small text-secondary"> &nbsp;Last checked on {{ date }}</span>
        <div class="d-flex gap-1 mt-2">
          <BBadge v-for="t in monitor.tags" variant="light">{{ t }}</BBadge>
        </div>
      </div>
      <BButtonToolbar class="align-self-start">
        <NuxtLink class="btn btn-sm btn-light icon-link">
          <Icon name="ph:eye-fill" />
          Details
        </NuxtLink>
      </BButtonToolbar>
    </BCardBody>
  </NuxtLink>
</template>

<style scoped lang="scss">
.btn-toolbar {
  display: none;
}

.card {
  cursor: pointer;
  text-decoration: inherit;
}

.card:hover {
  background-color: var(--body-bg-secondary);

  .btn-toolbar {
    display: unset;
  }
}
</style>