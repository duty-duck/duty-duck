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
  <BCard
    class="mb-3 shadow-sm"
    :class="{
      'border-danger': monitor.status == 'down',
      'border-warning': monitor.status == 'suspicious',
      'border-info': monitor.status == 'recovering',
      'border-secondary':
        monitor.status == 'unknown' || monitor.status == 'inactive',
      'border-success': monitor.status == 'up',
    }"
  >
    <div class="d-flex justify-content-between">
      <BCardTitle>
        {{ monitor.url }}
      </BCardTitle>
      <div>
        <BButtonToolbar>
          <NuxtLink class="btn btn-sm btn-light icon-link">
            <Icon name="ph:eye" />
            Details
          </NuxtLink>
        </BButtonToolbar>
      </div>
    </div>
    <p v-show="date" class="small text-secondary">Last checked on {{ date }}</p>
    <MonitorStatus :status="monitor.status" />
    <div class="d-flex gap-1 mt-2">
      <BBadge v-for="t in monitor.tags" variant="light">{{ t }}</BBadge>
    </div>
  </BCard>
</template>
